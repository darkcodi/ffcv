use crate::profile::{find_profile_path, get_prefs_path, list_profiles as list_profiles_impl};
use crate::query;
use crate::types::Config;

/// List all available Firefox profiles
pub fn list_profiles() -> Result<(), Box<dyn std::error::Error>> {
    let profiles = list_profiles_impl().map_err(|e| {
        anyhow::anyhow!(
            "Failed to list profiles: {}. Make sure Firefox is installed.",
            e
        )
    })?;

    let json = serde_json::to_string_pretty(&profiles)?;
    println!("{}", json);
    Ok(())
}

/// View configuration for a specific profile
pub fn view_config(
    profile_name: &str,
    query_patterns: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let profile_path = find_profile_path(profile_name).map_err(|e| {
        anyhow::anyhow!(
            "Failed to find profile '{}': {}. Make sure Firefox is installed and the profile exists.\n\
             Use 'ffcv profile' to see available profiles.",
            profile_name,
            e
        )
    })?;

    let prefs_path = get_prefs_path(&profile_path);
    let content = std::fs::read_to_string(&prefs_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to read prefs.js at {}: {e}. Make sure the file exists and is readable.",
            prefs_path.display()
        )
    })?;

    let preferences: Config = crate::parser::parse_prefs_js(&content).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse prefs.js: {e}. The file may be corrupted or in an unexpected format."
        )
    })?;

    // Apply queries if provided
    let output_config = if !query_patterns.is_empty() {
        query::query_preferences(&preferences, query_patterns)
            .map_err(|e| anyhow::anyhow!("Failed to apply query: {}", e))?
    } else {
        preferences
    };

    let json = serde_json::to_string_pretty(&output_config)?;
    println!("{}", json);
    Ok(())
}
