use anyhow::Result;
use chrono::prelude::*;

pub(crate) trait SolanaQueries {
    fn get_first_deployed_slot_timestamp(&self, program_id: &str) -> Result<DateTime<Utc>>;
}
