#![allow(unused_imports, unused_variables, dead_code)]

use anyhow::{anyhow, Result};
use tracing::{instrument, trace, Level};
use tracing_subscriber::{
    filter::{filter_fn, LevelFilter},
    fmt::{format::FmtSpan, writer::MakeWriterExt},
    prelude::*,
};

#[instrument]
pub(crate) fn init_tracing(debug_level: u8) -> Result<()> {
    let writer = std::io::stderr
        .with_max_level(Level::WARN)
        .or_else(std::io::stdout);

    tracing_subscriber::fmt()
        .with_writer(writer)
        .pretty()
        .with_max_level(as_level_filter(debug_level))
        .init();

    trace!(
        "Initializing tracing with debug level: {}",
        as_level_filter(debug_level).to_string().to_uppercase()
    );

    Ok(())
}

fn as_level_filter(debug_level: u8) -> LevelFilter {
    match debug_level {
        0 => LevelFilter::ERROR,
        1 => LevelFilter::WARN,
        2 => LevelFilter::INFO,
        3 => LevelFilter::DEBUG,
        _ => LevelFilter::TRACE,
    }
}
