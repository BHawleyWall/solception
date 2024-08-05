use anyhow::Result;
use chrono::prelude::*;
use tracing::{debug, instrument};

use crate::use_cases::SolanaQueries;

pub(crate) struct ProgramDataProvenance {
    solana: Box<dyn SolanaQueries>,
}

impl ProgramDataProvenance {
    #[instrument(skip(solana))]
    pub fn new(solana: Box<dyn SolanaQueries>) -> Self {
        Self { solana }
    }

    #[instrument(skip(self))]
    pub fn lookup_provenance(&self, program_id: &str) -> Result<DateTime<Utc>> {
        debug!("Beginning program provenance via gateway lookup for {program_id}.");

        self.solana.get_first_deployed_slot_timestamp(program_id)
    }
}
