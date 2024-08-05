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

use crate::use_cases::SolanaQueries;

pub(crate) struct SolanaRpc {
    rpc_client: RpcClient,
}

impl SolanaRpc {
    fn new(rpc_client: RpcClient) -> Self {
        Self { rpc_client }
    }

    pub fn new_with_url(rpc_url: &str) -> Self {
        let rpc_client = RpcClient::new(rpc_url.to_string());

        Self::new(rpc_client)
    }

    pub fn new_with_timeout(rpc_url: &str, timeout: u64) -> Self {
        let rpc_client = RpcClient::new_with_timeout(
            rpc_url.to_string(),
            std::time::Duration::from_secs(timeout),
        );

        Self::new(rpc_client)
    }
}

impl SolanaQueries for SolanaRpc {
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
        println!(
            "Retrieved {} transactions for {}",
            transactions.len(),
            program_id
        );

        if transactions.is_empty() {
            return Err(anyhow!(
                "No transactions found for program_id: {program_id}"
            ));
        }

        let txn_block_timestamp = transactions
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
            .filter(is_deployment)
            .min_by(|a, b| {
                a.block_time
                    .unwrap_or_default()
                    .cmp(&b.block_time.unwrap_or_default())
            })
            .expect(
                "No deployment details found for the given program ID.  Most likely this is an \
                 error in input for the address or the network, but the chosen RPC node could be \
                 missing historical data, or network issues prevented retrieval of any \
                 transaction details.  Check the program ID on a blockchain explorer to verify \
                 that it is valid on the chosen network, and has a transaction history with at \
                 least one BPFLoaderUpgradeab1e transaction.",
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

fn crawl_transaction_history(
    rpc_client: &RpcClient,
    program_id: &Pubkey,
) -> Result<Vec<RpcConfirmedTransactionStatusWithSignature>> {
    println!("Retrieving transactions for program_id: {}", program_id);

    const DEFAULT_SERVER_SIDE_LIMIT: usize = 1000;

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

        if batch_size < DEFAULT_SERVER_SIDE_LIMIT {
            println!("Exiting history crawl loop.");
            break;
        } else {
            println!("Continuing history crawl loop...");
        }
    }

    Ok(transactions)
}

fn is_deployment(rpc_txn: &EncodedConfirmedTransactionWithStatusMeta) -> bool {
    let encoded_txn = rpc_txn.transaction.to_owned();
    println!("\n_~* START *~_\n{:?}\n", encoded_txn);

    let json_txn = encoded_txn.transaction;
    println!("\n{:?}\n", json_txn);

    match json_txn {
        EncodedTransaction::Json(json) => is_deployment_json(json),
        _ => false,
    }
}

fn is_deployment_json(json: UiTransaction) -> bool {
    println!("\n{:?}\n", json);
    match json.message {
        UiMessage::Parsed(message) => is_deployment_message(message),
        _ => false,
    }
}

fn is_deployment_message(message: UiParsedMessage) -> bool {
    println!("\n{:?}\n_~* END *~_\n", message);

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
