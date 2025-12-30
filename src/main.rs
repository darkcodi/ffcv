mod cli;
mod commands;
mod parser;
mod profile;
mod types;

use clap::Parser;
use cli::Cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        cli::Commands::Profile => commands::list_profiles(),
        cli::Commands::Config { profile } => commands::view_config(&profile),
    }
}
