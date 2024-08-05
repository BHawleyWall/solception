use anyhow::Result;
use tracing::{debug, instrument};

use crate::{adapters::controllers::ProvenanceAdapter, use_cases::SolanaQueries};

pub(crate) struct ProvenanceToCli {
    adapter: ProvenanceAdapter,
}

impl ProvenanceToCli {
    #[instrument(skip(adapter))]
    pub fn new(adapter: ProvenanceAdapter) -> Self {
        Self { adapter }
    }

    #[instrument(skip(solana))]
    pub fn new_with_gateway(solana: Box<dyn SolanaQueries>) -> Self {
        let adapter = ProvenanceAdapter::new_with_gateway(solana);

        Self::new(adapter)
    }

    #[instrument(skip(self))]
    pub fn lookup_provenance(&self, program_id: &str) -> Result<String> {
        debug!("Beginning program provenance via adapter lookup for {program_id}.");

        let timestamp = self.adapter.lookup_provenance(program_id)?;

        debug!("Provenance lookup complete.  Returning as RFC 3339 timestamp for CLI stdout.");
        Ok(timestamp.to_rfc3339())
    }
}
