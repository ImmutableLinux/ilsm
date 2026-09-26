mod cli;
mod assemble;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Immutable Linux System Manager")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    ///Assemble a new immutable Linux system image
    Assemble
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Assemble => {
            println!("Not yet implemented: Assemble command");
        }
    }
}
