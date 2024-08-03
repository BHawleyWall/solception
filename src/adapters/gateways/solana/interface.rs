#![allow(unused_imports, unused_variables, dead_code)]

use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use chrono::prelude::*;
use rayon::prelude::*;
use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
use solana_sdk::{
    bpf_loader_upgradeable::UpgradeableLoaderState,
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::Signature,
    slot_history::Slot,
    transaction::Transaction,
};
use solana_transaction_status::{
    parse_accounts::ParsedAccount,
    EncodedTransaction,
    UiMessage,
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

        let bpf_loader_upgradeable =
            Pubkey::from_str("BPFLoaderUpgradeab1e11111111111111111111111").expect(
                "This Solana system program ID failed to parse.  This should only occur if the \
                 Solana Labs crates have changed the system program ID.  Check the Git blame for \
                 the Solana Rust SDK to see if related changes were made to the Pubkey object's \
                 parser or the system IDs.",
            );

        let transactions = self.rpc_client.get_signatures_for_address_with_config(
            &program_id,
            GetConfirmedSignaturesForAddress2Config {
                before: None,
                until: None,
                limit: None,
                commitment: Some(CommitmentConfig::finalized()),
            },
        )?;

        let txn_block_timestamp = transactions
            .par_iter()
            .filter_map(|txn| {
                let sig = Signature::from_str(&txn.signature).unwrap();
                self.rpc_client
                    .get_transaction(&sig, UiTransactionEncoding::JsonParsed)
                    .ok()
            })
            .filter(|txn| {
                let encoded_with_meta_txn = txn.transaction.to_owned();
                println!("\n_~* START *~_\n{:?}\n", encoded_with_meta_txn);
                let json_ui_txn = encoded_with_meta_txn.transaction;
                println!("\n{:?}\n", json_ui_txn);
                match json_ui_txn {
                    EncodedTransaction::Json(json) => {
                        println!("\n{:?}\n", json);
                        match json.message {
                            UiMessage::Parsed(message) => {
                                println!("\n{:?}\n_~* END *~_\n", message);
                                let acct_keys = message
                                    .account_keys
                                    .iter()
                                    .map(|parsed_acct| parsed_acct.pubkey.to_string())
                                    .collect::<Vec<_>>();
                                acct_keys
                                    .iter()
                                    .any(|acct_key| acct_key == &bpf_loader_upgradeable.to_string())
                            }
                            _ => false,
                        }
                    }
                    _ => false,
                }
            })
            .min_by(|a, b| {
                a.block_time
                    .unwrap_or_default()
                    .cmp(&b.block_time.unwrap_or_default())
            })
            .unwrap()
            .block_time
            .unwrap();
        let txn_block_time = DateTime::from_timestamp(txn_block_timestamp, 0).unwrap();

        Ok(txn_block_time)
    }
}
