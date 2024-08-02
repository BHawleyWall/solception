#![allow(unused_imports, unused_variables, dead_code)]

use anyhow::{anyhow, Context, Result};
use chrono::prelude::*;

#[async_trait::async_trait]
pub(crate) trait SolanaQueries {
    async fn get_first_deployed_slot_timestamp(&self, program_id: &str) -> Result<DateTime<Utc>>;
}
