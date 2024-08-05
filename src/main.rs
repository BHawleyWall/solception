use clap::Parser;
use solception::lookup_provenance;

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Cli {
    /// Required argument for the target program on Solana in the form of its
    /// base 58 public key.
    program_id: String,
}

fn main() {
    let cli = Cli::parse();

    let timestamp = lookup_provenance("https://api.devnet.solana.com", &cli.program_id)
        .unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        });

    println!("{}", timestamp);

    std::process::exit(0);
}
