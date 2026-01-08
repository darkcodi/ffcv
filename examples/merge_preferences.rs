// Example: Merge Firefox Preferences
//
// This example demonstrates how to merge preferences from multiple sources
// (built-in defaults, global defaults, and user preferences) using ffcv.

use ffcv::{find_profile_path, merge_all_preferences, MergeConfig};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get profile name from command line args, or use default
    let profile_name = env::args()
        .nth(1)
        .unwrap_or_else(|| "default-release".to_string());

    println!(
        "Merging Firefox Preferences for profile: {}\n",
        profile_name
    );

    // Find the profile path
    let profile_path = find_profile_path(&profile_name, None)?;

    println!("Profile path: {}", profile_path.display());

    // Configure the merge
    let config = MergeConfig {
        include_builtins: true,
        include_globals: true,
        include_user: true,
        continue_on_error: true,
    };

    println!("\nMerging preferences...");
    println!("  Built-in defaults (omni.ja): {}", config.include_builtins);
    println!(
        "  Global defaults (greprefs.js): {}",
        config.include_globals
    );
    println!("  User preferences (prefs.js): {}", config.include_user);

    // Perform the merge
    let merged = merge_all_preferences(&profile_path, None, &config)?;

    println!("\nMerge Results:");
    println!("  Total preferences: {}", merged.entries.len());
    println!("  Loaded sources: {:?}", merged.loaded_sources);

    if let Some(install_path) = &merged.install_path {
        println!("  Installation path: {}", install_path.display());
    }

    // Show warnings if any
    if !merged.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &merged.warnings {
            println!("  ⚠ {}", warning);
        }
    }

    // Show some example preferences
    println!("\nExample Preferences:");
    println!("====================");

    let examples = [
        "browser.startup.homepage",
        "network.proxy.type",
        "privacy.trackingprotection.enabled",
        "layout.css.devPixelsPerPx",
    ];

    for key in &examples {
        if let Some(pref) = merged.entries.iter().find(|e| e.key == *key) {
            println!("\n{}:", pref.key);
            println!("  Value: {:?}", pref.value);
            println!("  Type: {:?}", pref.pref_type);
            println!("  Source: {:?}", pref.source);
        }
    }

    Ok(())
}
