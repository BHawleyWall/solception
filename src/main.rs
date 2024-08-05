#![allow(unused_imports, unused_variables, dead_code)]

use clap::{ArgAction, Parser};
use solception::lookup_provenance;

const PUBLIC_DEVNET_RPC_NODE_URL: &str = "https://api.devnet.solana.com";
const PUBLIC_MAINNET_BETA_RPC_NODE_URL: &str = "https://api.mainnet-beta.solana.com";

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Cli {
    /// Required argument for the target program on Solana in the form of its
    /// base 58 public key.
    program_id: String,

    /// Optional logging verbosity level. Repeat up to four times to increase
    /// verbosity.
    #[arg(short, long, action=ArgAction::Count)]
    verbose: u8,
}

fn main() {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    let timestamp = lookup_provenance(
        cli.verbose,
        "https://api.devnet.solana.com",
        &cli.program_id,
    )
    .unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    });

    println!("{}", timestamp);

    std::process::exit(0);
}
