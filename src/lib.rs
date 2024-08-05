mod adapters;
mod entities;
mod use_cases;

use anyhow::Result;
use tracing::{instrument, trace};

use crate::adapters::{
    gateways::{solana::SolanaRpc, telemetry::init_tracing},
    presenters::ProvenanceToCli,
};

#[instrument]
pub fn lookup_provenance(debug_level: u8, node_url: &str, program_id: &str) -> Result<String> {
    init_tracing(debug_level)?;
    trace!("Entering library bootstrap path.");

    let solana = SolanaRpc::new_with_url(node_url);
    let presenter = ProvenanceToCli::new_with_gateway(Box::new(solana));
    trace!("Bootstrap complete.  Forwarding to presenter.");

    presenter.lookup_provenance(program_id)
}
