use anyhow::Result;
use chrono::prelude::*;
use tracing::{debug, instrument};

use crate::use_cases::{ProgramDataProvenance, SolanaQueries};

pub(crate) struct ProvenanceAdapter {
    use_case: ProgramDataProvenance,
}

impl ProvenanceAdapter {
    #[instrument(skip(use_case))]
    pub fn new(use_case: ProgramDataProvenance) -> Self {
        Self { use_case }
    }

    #[instrument(skip(solana))]
    pub fn new_with_gateway(solana: Box<dyn SolanaQueries>) -> Self {
        let use_case = ProgramDataProvenance::new(solana);

        Self::new(use_case)
    }

    #[instrument(skip(self))]
    pub fn lookup_provenance(&self, program_id: &str) -> Result<DateTime<Utc>> {
        debug!("Beginning program provenance via use case lookup for {program_id}.");

        self.use_case.lookup_provenance(program_id)
    }
}
