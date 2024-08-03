mod adapters;
mod entities;
mod use_cases;

use anyhow::Result;

use crate::adapters::{gateways::solana::SolanaRpc, presenters::ProvenanceToCli};

pub fn lookup_provenance(node_url: &str, program_id: &str) -> Result<String> {
    let solana = SolanaRpc::new_with_url(node_url);
    let presenter = ProvenanceToCli::new_with_gateway(Box::new(solana));

    presenter.lookup_provenance(program_id)
}
