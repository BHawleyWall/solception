#![allow(unused_imports, unused_variables, dead_code)]

use anyhow::{anyhow, Context, Result};
use chrono::prelude::*;

use crate::use_cases::SolanaQueries;

pub(crate) struct ProgramDataProvenance {
    solana: Box<dyn SolanaQueries>,
}

impl ProgramDataProvenance {
    pub fn new(solana: Box<dyn SolanaQueries>) -> Self {
        Self { solana }
    }

    pub fn lookup_provenance(&self, program_id: &str) -> Result<DateTime<Utc>> {
        self.solana.get_first_deployed_slot_timestamp(program_id)
    }
}
