// Example: Filter Modified Preferences
//
// This example demonstrates how to filter and show only user-modified
// preferences (those that differ from Firefox's built-in defaults).

use ffcv::{find_profile_path, merge_all_preferences, MergeConfig, PrefSource};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profile_name = "default-release";
    println!("Finding user-modified preferences for: {}\n", profile_name);

    // Find the profile path
    let profile_path = find_profile_path(profile_name, None)?;

    // Merge all preferences
    let config = MergeConfig {
        include_builtins: true,
        include_globals: true,
        include_user: true,
        continue_on_error: true,
    };

    let merged = merge_all_preferences(&profile_path, None, &config)?;

    // Filter to only user-modified preferences
    let user_modified: Vec<_> = merged
        .entries
        .iter()
        .filter(|pref| pref.source == Some(PrefSource::User))
        .collect();

    println!("Found {} user-modified preferences:\n", user_modified.len());

    // Group by category
    let mut network_prefs = Vec::new();
    let mut browser_prefs = Vec::new();
    let mut privacy_prefs = Vec::new();
    let mut other_prefs = Vec::new();

    for pref in &user_modified {
        if pref.key.starts_with("network.") {
            network_prefs.push(*pref);
        } else if pref.key.starts_with("browser.") {
            browser_prefs.push(*pref);
        } else if pref.key.starts_with("privacy.") {
            privacy_prefs.push(*pref);
        } else {
            other_prefs.push(*pref);
        }
    }

    // Display preferences by category
    display_category("Network", network_prefs);
    display_category("Browser", browser_prefs);
    display_category("Privacy", privacy_prefs);
    display_category("Other", other_prefs);

    Ok(())
}

fn display_category(name: &str, prefs: Vec<&ffcv::PrefEntry>) {
    if prefs.is_empty() {
        return;
    }

    println!("{} Preferences:", name);
    println!("{}:", "-".repeat(40));

    // Sort by key
    let mut sorted: Vec<_> = prefs.into_iter().collect();
    sorted.sort_by_key(|p| &p.key);

    for pref in sorted {
        println!("  {:50} {:?}", pref.key, pref.value);
    }
    println!();
}
