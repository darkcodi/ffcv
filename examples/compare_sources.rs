// Example: Compare Preference Sources
//
// This example demonstrates how to compare the same preference across
// different sources (built-in defaults, global defaults, user preferences)
// to understand how Firefox merges and prioritizes preferences.

use ffcv::{find_profile_path, merge_all_preferences, MergeConfig, PrefSource};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profile_name = "default-release";
    println!("Comparing Preference Sources\n");
    println!("This shows how the same preference might have different values");
    println!("in different sources, with user preferences taking precedence.\n");

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

    // Track preferences by key to find those that appear in multiple sources
    use std::collections::HashMap;

    let mut by_key: HashMap<String, Vec<&ffcv::PrefEntry>> = HashMap::new();

    for pref in &merged.entries {
        by_key.entry(pref.key.clone()).or_default().push(pref);
    }

    // Find preferences that have multiple sources
    let mut multi_source = Vec::new();

    for (key, prefs) in &by_key {
        if prefs.len() > 1 {
            multi_source.push((key.clone(), prefs.clone()));
        }
    }

    if multi_source.is_empty() {
        println!("No preferences found in multiple sources.");
        println!("All preferences have unique sources.");
    } else {
        println!(
            "Found {} preferences in multiple sources:\n",
            multi_source.len()
        );

        // Sort by key
        multi_source.sort_by_key(|(key, _)| key.clone());

        for (key, prefs) in multi_source.iter().take(20) {
            println!("{}:", key);
            for pref in prefs {
                let source_name = match pref.source {
                    Some(PrefSource::BuiltIn) => "Built-in",
                    Some(PrefSource::GlobalDefault) => "Global",
                    Some(PrefSource::User) => "User",
                    Some(PrefSource::SystemPolicy) => "Policy",
                    None => "Unknown",
                };
                println!("  {:12} {:?}", source_name, pref.value);
            }
            println!();
        }
    }

    // Show summary
    println!("Summary:");
    println!("  Total preferences: {}", merged.entries.len());

    let source_counts = [
        (PrefSource::BuiltIn, "Built-in defaults"),
        (PrefSource::GlobalDefault, "Global defaults"),
        (PrefSource::User, "User preferences"),
    ];

    for (source, name) in source_counts {
        let count = merged
            .entries
            .iter()
            .filter(|p| p.source.as_ref() == Some(&source))
            .count();
        println!("  {}: {}", name, count);
    }

    Ok(())
}
