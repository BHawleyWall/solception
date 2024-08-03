use anyhow::Result;
use chrono::prelude::*;

use crate::use_cases::{ProgramDataProvenance, SolanaQueries};

pub(crate) struct ProvenanceAdapter {
    use_case: ProgramDataProvenance,
}

impl ProvenanceAdapter {
    pub fn new(use_case: ProgramDataProvenance) -> Self {
        Self { use_case }
    }

    pub fn new_with_gateway(solana: Box<dyn SolanaQueries>) -> Self {
        let use_case = ProgramDataProvenance::new(solana);

        Self::new(use_case)
    }

    pub fn lookup_provenance(&self, program_id: &str) -> Result<DateTime<Utc>> {
        self.use_case.lookup_provenance(program_id)
    }
}
