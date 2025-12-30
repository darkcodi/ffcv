use clap::{Parser, Subcommand};

/// View Firefox configuration from the command line
#[derive(Parser, Debug)]
#[command(name = "ffcv")]
#[command(about = "View Firefox configuration from the command line")]
#[command(
    long_about = "ffcv lets you view Firefox configuration (prefs.js) as JSON \
from the command line. Use subcommands to list profiles or inspect configuration."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all available Firefox profiles
    Profile,

    /// View Firefox configuration for a profile
    Config {
        /// Firefox profile name (default: "default")
        #[arg(short = 'p', long, default_value = "default")]
        profile: String,
    },
}
