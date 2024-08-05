#![allow(unused_imports, unused_variables, dead_code)]

use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use chrono::prelude::*;
use lazy_static::lazy_static;
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_client::GetConfirmedSignaturesForAddress2Config,
    rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::{
    bpf_loader_upgradeable::UpgradeableLoaderState, commitment_config::CommitmentConfig,
    pubkey::Pubkey, signature::Signature, slot_history::Slot, transaction::Transaction,
};
use solana_transaction_status::{
    parse_accounts::ParsedAccount, EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
    EncodedTransactionWithStatusMeta, UiMessage, UiParsedMessage, UiTransaction,
    UiTransactionEncoding,
};
use tokio::{
    runtime::{Builder, Handle},
    sync::OnceCell,
    task::{JoinHandle, JoinSet},
};
use tokio_utils::RateLimiter;

use crate::use_cases::SolanaQueries;

static RPC_CLIENT: OnceCell<RpcClient> = OnceCell::const_new();

lazy_static! {
    static ref RATE_LIMITER: RateLimiter = RateLimiter::new(std::time::Duration::from_millis(300));
}

pub(crate) struct SolanaRpc {
    rpc_client: &'static RpcClient,
}

impl SolanaRpc {
    fn new(rpc_client: &'static RpcClient) -> Self {
        Self { rpc_client }
    }

    pub fn new_with_url(rpc_url: &str) -> Self {
        RPC_CLIENT.set(RpcClient::new(rpc_url.to_string())).ok();

        Self::new(RPC_CLIENT.get().expect(
            "Failed to retrieve lazy set client reference completed immediately before this \
             within this same constructor.  This should not be possible, and would only occur if \
             changes were made to this module.  Check the Git blame for clues.",
        ))
    }

    pub fn new_with_timeout(rpc_url: &str, timeout: u64) -> Self {
        RPC_CLIENT
            .set(RpcClient::new_with_timeout(
                rpc_url.to_string(),
                std::time::Duration::from_secs(timeout),
            ))
            .ok();

        Self::new(RPC_CLIENT.get().expect(
            "Failed to retrieve lazy set client reference completed immediately before this \
             within this same constructor.  This should not be possible, and would only occur if \
             changes were made to this module.  Check the Git blame for clues.",
        ))
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

        let runtime = Builder::new_multi_thread().enable_all().build()?;

        let transactions =
            crawl_transaction_history(runtime.handle(), self.rpc_client, &program_id)?;
        println!(
            "Retrieved {} transactions for {}",
            transactions.len(),
            program_id
        );

        if transactions.is_empty() {
            return Err(anyhow!(
                "No transactions found for program_id: {program_id} .  This could be due to a \
                 lack of transactions on the network, or the chosen RPC node could be missing \
                 historical data, or network issues prevented retrieval of any transaction \
                 details. Check the program ID on a blockchain explorer to verify that it is \
                 valid on the chosen network, and has a transaction history with at least one \
                 BPFLoaderUpgradeab1e transaction."
            ));
        }

        let txn_results = runtime
            .block_on(async move { get_all_transactions(self.rpc_client, transactions).await });

        let is_deployment_count = txn_results.iter().filter(|txn| is_deployment(txn)).count();
        println!("Deployment count: {}", is_deployment_count);

        let txn_block_timestamp = txn_results
            .iter()
            .filter(|txn| is_deployment(txn))
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
    runtime: &Handle,
    rpc_client: &RpcClient,
    program_id: &Pubkey,
) -> Result<Vec<RpcConfirmedTransactionStatusWithSignature>> {
    println!("Retrieving transactions for program_id: {}", program_id);

    const DEFAULT_SERVER_SIDE_LIMIT: usize = 1000;

    let mut transactions: Vec<RpcConfirmedTransactionStatusWithSignature> = Vec::new();
    let mut before_sig_opt: Option<Signature> = None;

    let mut loop_counter = 0;
    println!("Entering history crawl loop...");
    loop {
        let batch = runtime.block_on(async move {
            rpc_client
                .get_signatures_for_address_with_config(
                    program_id,
                    GetConfirmedSignaturesForAddress2Config {
                        before: before_sig_opt,
                        until: None,
                        limit: None,
                        commitment: Some(CommitmentConfig::finalized()),
                    },
                )
                .await
        })?;

        let batch_size = batch.len();
        before_sig_opt = batch
            .last()
            .map(|txn| Signature::from_str(&txn.signature).unwrap());

        transactions.extend(batch);

        if batch_size < DEFAULT_SERVER_SIDE_LIMIT {
            println!("Exiting history crawl loop.");
            break;
        } else {
            loop_counter += 1;
            println!("{}", ".".repeat(loop_counter));
        }
    }

    Ok(transactions)
}

async fn get_all_transactions(
    rpc_client: &'static RpcClient,
    transactions: Vec<RpcConfirmedTransactionStatusWithSignature>,
) -> Vec<EncodedConfirmedTransactionWithStatusMeta> {
    println!(
        "Retrieving transaction details for {} transactions",
        transactions.len()
    );
    let mut txn_queries = JoinSet::new();

    for transaction in transactions {
        RATE_LIMITER
            .throttle(|| async {
                txn_queries.spawn(get_transaction(
                    rpc_client,
                    transaction.signature.to_owned(),
                ))
            })
            .await;
    }

    let mut results = Vec::new();

    while let Some(result) = txn_queries.join_next().await {
        match result {
            Ok(Ok(txn)) => {
                let txn_time = txn.block_time.unwrap_or_default();
                results.push(txn);
                println!("Fan-out task succeeded: {txn_time}");
            }
            Ok(Err(e)) => {
                if e.source()
                    .unwrap()
                    .to_string()
                    .contains("429 Too Many Requests")
                {
                    eprintln!("Rate limit exceeded.  Retrying...");
                    RATE_LIMITER
                        .throttle(|| async {
                            txn_queries.spawn(get_transaction(
                                rpc_client,
                                e.to_string().split_whitespace().last().unwrap().to_string(),
                            ));
                        })
                        .await;
                } else {
                    eprintln!("RPC API Error: {e}");
                }
            }
            _ => {
                eprintln!("Fan-out task Error: {result:?}");
            }
        }
    }

    println!(
        "Retrieved transaction details for {} transactions",
        results.len()
    );
    results
}

async fn spawn_rate_limited_task<F, T>(rate_limiter: &RateLimiter, task: F) -> T
where
    F: FnOnce() -> T + Send,
    T: Send,
{
    rate_limiter.throttle(|| async { task() }).await
}

async fn get_transaction(
    rpc_client: &RpcClient,
    signature: String,
) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
    let sig = Signature::from_str(signature.as_str()).unwrap();
    rpc_client
        .get_transaction(&sig, UiTransactionEncoding::JsonParsed)
        .await
        .with_context(|| {
            format!("Failed to retrieve transaction details for signature: {signature}")
        })
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
