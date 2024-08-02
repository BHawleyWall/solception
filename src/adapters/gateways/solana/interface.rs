#![allow(unused_imports, unused_variables, dead_code)]

/* steps
 * 1. lookup the program id on the blockchain and get a Program Account object
 * 2. get the Executable Data account from the Program Account object
 * 3. lookup the Executable Data account on the blockchain and get a Program Executable Data
 *    Account object
 * 4. get the Last Deployed Slot from the Program Executable Data Account object
 * 5. lookup the Last Deployed Slot on the blockchain and get a Slot object
 * 6. get all Transactions in the slot interacting with the BPF Upgradeable Loader system program
 *    and the Program Account
 * 7  get the timestamp of the youngest Transaction in that result set (prolly only 1)
 */

use anyhow::{anyhow, Context, Result};
use chrono::prelude::*;
use solana_client::{
    rpc_client::RpcClient, rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::{
    pubkey::Pubkey, signature::Signature, slot_history::Slot, transaction::Transaction,
};

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
