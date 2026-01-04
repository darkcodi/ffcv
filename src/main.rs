//! Firefox Configuration Viewer - Command-line interface
//!
//! This binary provides a command-line tool for parsing and querying
//! Firefox preference files. It supports listing Firefox profiles,
//! viewing configuration preferences, and querying preferences using
//! glob patterns.
//!
//! # Subcommands
//!
//! - `ffcv profile list` - List all Firefox profiles on the system
//! - `ffcv config view <profile>` - View configuration for a profile
//! - `ffcv config get <profile> <key>` - Get a specific preference value
//!
//! For programmatic usage, see the [library documentation](../ffcv/index.html).
//!
//! # Examples
//!
//! ```bash
//! # List all Firefox profiles
//! ffcv profile list
//!
//! # View all preferences for the default profile
//! ffcv config view default
//!
//! # Query network-related preferences
//! ffcv config view default --query "network.*"
//!
//! # Get a single preference
//! ffcv config get default "network.proxy.type"
//! ```

mod cli;
mod commands;

use clap::Parser;
use cli::Cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        cli::Commands::Profile { profiles_dir } => commands::list_profiles(profiles_dir.as_deref()),
        cli::Commands::Config {
            profile,
            profiles_dir,
            stdin,
            max_file_size,
            query,
            get,
            output_type,
            unexplained_only,
        } => {
            // Convert Vec<String> to Vec<&str> for query_preferences
            let query_refs: Vec<&str> = query.iter().map(|s| s.as_str()).collect();
            commands::view_config(commands::ViewConfigParams {
                stdin,
                profile_name: &profile,
                profiles_dir_opt: profiles_dir.as_deref(),
                max_file_size,
                query_patterns: &query_refs,
                get,
                output_type,
                unexplained_only,
            })
        }
    }
}
