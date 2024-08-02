use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Cli {
    /// Required argument for the target program on Solana
    program_id: String,
}

fn main() {
    let _cli = Cli::parse();

    std::process::exit(1);
}
