mod cli;
mod parser;
mod profile;
mod types;

use cli::Args;
use profile::{find_profile_path, get_prefs_path};
use types::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let args = Args::parse();

    // Find the profile directory
    let profile_path = find_profile_path(&args.profile)
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to find profile '{}': {}. Make sure Firefox is installed and the profile exists.",
                args.profile,
                e
            )
        })?;

    // Get the prefs.js file path
    let prefs_path = get_prefs_path(&profile_path);

    // Read the prefs.js file
    let content = std::fs::read_to_string(&prefs_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to read prefs.js at {}: {}. Make sure the file exists and is readable.",
            prefs_path.display(),
            e
        )
    })?;

    // Parse the preferences
    let preferences: Config = parser::parse_prefs_js(&content).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse prefs.js: {}. The file may be corrupted or in an unexpected format.",
            e
        )
    })?;

    // Output as pretty-printed JSON
    let json = serde_json::to_string_pretty(&preferences)?;
    println!("{}", json);

    Ok(())
}
