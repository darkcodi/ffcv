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
