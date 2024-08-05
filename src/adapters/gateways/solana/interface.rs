use std::str::FromStr;

use anyhow::{anyhow, Result};
use chrono::prelude::*;
use rayon::prelude::*;
use solana_client::{
    rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient},
    rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta,
    EncodedTransaction,
    UiMessage,
    UiParsedMessage,
    UiTransaction,
    UiTransactionEncoding,
};
use tracing::{debug, info, instrument, trace, warn};

use crate::use_cases::SolanaQueries;

const DEFAULT_SERVER_SIDE_BATCH_LIMIT: usize = 1000;

pub(crate) struct SolanaRpc {
    rpc_client: RpcClient,
}

impl SolanaRpc {
    #[instrument(skip(rpc_client))]
    fn new(rpc_client: RpcClient) -> Self {
        Self { rpc_client }
    }

    #[instrument]
    pub fn new_with_url(rpc_url: &str) -> Self {
        let rpc_client = RpcClient::new(rpc_url.to_string());

        Self::new(rpc_client)
    }

    #[instrument]
    pub fn new_with_timeout(rpc_url: &str, timeout: u64) -> Self {
        let rpc_client = RpcClient::new_with_timeout(
            rpc_url.to_string(),
            std::time::Duration::from_secs(timeout),
        );

        Self::new(rpc_client)
    }
}

impl SolanaQueries for SolanaRpc {
    #[instrument(skip(self))]
    fn get_first_deployed_slot_timestamp(&self, program_id: &str) -> Result<DateTime<Utc>> {
        let program_id = Pubkey::from_str(program_id).map_err(|e| {
            anyhow!(
                "Failed to parse program_id: {} .  Most likely the provided value is not a base \
                 58 public key.  Check the input against a blockchain explorer, and if it is \
                 valid, check the Git blame for the Solana Rust SDK to see if related changes \
                 were made to the Pubkey object's parser.",
                e
            )
        })?;

        let transactions = crawl_transaction_history(&self.rpc_client, &program_id)?;
        debug!(
            "Retrieved {} transaction summaries for {}",
            transactions.len(),
            program_id
        );

        if transactions.is_empty() {
            return Err(anyhow!(
                "No transactions found for program_id: {program_id}"
            ));
        }

        info!(
            "Retrieving transaction details for program_id: {program_id}.  This may take some \
             time, depending on the number of transactions and the chosen cluster RPC node's rate \
             limits."
        );

        if transactions.len() > DEFAULT_SERVER_SIDE_BATCH_LIMIT {
            warn!(
                "The number of transactions for program_id: {program_id} exceeds 1000.  This may \
                 take a long time to retrieve all transaction details, depending on the chosen \
                 cluster RPC node's rate limits!"
            );
        }

        let txn_details = transactions
            .par_iter()
            .filter_map(|txn| {
                let sig = Signature::from_str(&txn.signature).expect(
                    "Failed to parse transaction signature taken directly from RPC response \
                     content.  This should only occur if this module's codepath was changed or \
                     the Solana Labs crates have changed the signature format.  Check the Git \
                     blame for this module first and then the Solana Rust SDK to see if related \
                     changes were made to the Signature object's parser.",
                );
                self.rpc_client
                    .get_transaction(&sig, UiTransactionEncoding::JsonParsed)
                    .ok()
            })
            .collect::<Vec<_>>();

        debug!(
            "Retrieved {} transaction details for {}",
            txn_details.len(),
            program_id
        );

        if transactions.len() != txn_details.len() {
            warn!(
                "Transaction details count does not match transaction summary count.  This should \
                 not be possible, as the RPC node should be able to return a transaction detail \
                 for every transaction summary.  Check the trace logs for actual HTTP return \
                 codes on each attempt."
            );
        }

        let deployments = txn_details
            .par_iter()
            .filter(|txn| is_deployment(txn))
            .collect::<Vec<_>>();

        debug!("Found {} deployments for {}", deployments.len(), program_id);

        let txn_block_timestamp = deployments
            .par_iter()
            .min_by(|a, b| {
                a.block_time
                    .unwrap_or_default()
                    .cmp(&b.block_time.unwrap_or_default())
            })
            .expect(
                "No deployment details found for the given program ID's history.  Most likely \
                 this is an error in input for the address or the network, but the chosen RPC \
                 node could be missing historical data, or network issues prevented retrieval of \
                 some transaction details.  Check the program ID on a blockchain explorer to \
                 verify that it is valid on the chosen network, and has a transaction history \
                 with at least one BPFLoaderUpgradeab1e transaction.",
            )
            .block_time
            .expect(
                "No block timestamp found on the oldest deployment transaction.  This should not \
                 be possible, as all finalized transactions have a block timestamp.  Double chack \
                 the RPC node's historical data using an explorer, and check the Git history for \
                 the Solana RPC library to see if there have been changes to the structures of \
                 the transaction data.",
            );
        let txn_block_time = DateTime::from_timestamp(txn_block_timestamp, 0).unwrap();

        Ok(txn_block_time)
    }
}

#[instrument(skip(rpc_client))]
fn crawl_transaction_history(
    rpc_client: &RpcClient,
    program_id: &Pubkey,
) -> Result<Vec<RpcConfirmedTransactionStatusWithSignature>> {
    debug!("Retrieving transaction details for {program_id}");

    let mut transactions: Vec<RpcConfirmedTransactionStatusWithSignature> = Vec::new();
    let mut before_sig_opt: Option<Signature> = None;

    loop {
        let batch = rpc_client.get_signatures_for_address_with_config(
            program_id,
            GetConfirmedSignaturesForAddress2Config {
                before: before_sig_opt,
                until: None,
                limit: None,
                commitment: Some(CommitmentConfig::finalized()),
            },
        )?;

        let batch_size = batch.len();
        before_sig_opt = batch
            .last()
            .map(|txn| Signature::from_str(&txn.signature).unwrap());

        transactions.extend(batch);

        if batch_size < DEFAULT_SERVER_SIDE_BATCH_LIMIT {
            trace!("Exiting history crawl loop.");
            break;
        } else {
            trace!("Continuing history crawl loop...");
        }
    }

    Ok(transactions)
}

#[instrument]
fn is_deployment(rpc_txn: &EncodedConfirmedTransactionWithStatusMeta) -> bool {
    let encoded_txn = rpc_txn.transaction.to_owned();
    trace!("{:?}", encoded_txn);

    let json_txn = encoded_txn.transaction;
    trace!("{:?}", json_txn);

    match json_txn {
        EncodedTransaction::Json(json) => is_deployment_json(json),
        _ => false,
    }
}
#[instrument]
fn is_deployment_json(json: UiTransaction) -> bool {
    trace!("{:?}", json);

    debug!(
        "Checking if transaction {} is a deployment...",
        json.signatures.first().unwrap()
    );

    match json.message {
        UiMessage::Parsed(message) => is_deployment_message(message),
        _ => false,
    }
}

#[instrument]
fn is_deployment_message(message: UiParsedMessage) -> bool {
    trace!("{:?}", message);

    let bpf_loader_upgradeable = Pubkey::from_str("BPFLoaderUpgradeab1e11111111111111111111111")
        .expect(
            "This Solana system program ID failed to parse.  This should only occur if the Solana \
             Labs crates have changed the system program ID.  Check the Git blame for the Solana \
             Rust SDK to see if related changes were made to the Pubkey object's parser or the \
             system IDs.",
        );

    let acct_keys = message
        .account_keys
        .iter()
        .map(|parsed_acct| parsed_acct.pubkey.to_string())
        .collect::<Vec<_>>();
    acct_keys
        .iter()
        .any(|acct_key| acct_key == &bpf_loader_upgradeable.to_string())
}
