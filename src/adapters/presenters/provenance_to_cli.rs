use anyhow::Result;

use crate::{adapters::controllers::ProvenanceAdapter, use_cases::SolanaQueries};

pub(crate) struct ProvenanceToCli {
    adapter: ProvenanceAdapter,
}

impl ProvenanceToCli {
    pub fn new(adapter: ProvenanceAdapter) -> Self {
        Self { adapter }
    }

    pub fn new_with_gateway(solana: Box<dyn SolanaQueries>) -> Self {
        let adapter = ProvenanceAdapter::new_with_gateway(solana);

        Self::new(adapter)
    }

    pub fn lookup_provenance(&self, program_id: &str) -> Result<String> {
        let timestamp = self.adapter.lookup_provenance(program_id)?;

        Ok(timestamp.to_rfc3339())
    }
}
