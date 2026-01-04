//! Basic parsing example for Firefox preference files
//!
//! This example demonstrates how to parse a Firefox prefs.js file
//! and display the parsed preferences.

use ffcv::{parse_prefs_js, PrefType, PrefValue};
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the prefs.js file path from command line args, or use a default
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 {
        Path::new(&args[1])
    } else {
        eprintln!("Usage: {} <prefs.js path>", args[0]);
        eprintln!("Example: {} /path/to/firefox/default/prefs.js", args[0]);
        return Ok(());
    };

    // Read the prefs.js file
    println!("Reading preferences from: {}", path.display());
    let content = fs::read_to_string(path)?;

    // Parse the preferences
    let entries = parse_prefs_js(&content)?;

    println!("\nFound {} preference entries\n", entries.len());
    println!("{:-<80}", "");
    println!("{:<40} {:<10} {:<20}", "Preference", "Type", "Value");
    println!("{:-<80}", "");

    // Display each entry
    for entry in &entries {
        let type_name = match entry.pref_type {
            PrefType::User => "User",
            PrefType::Default => "Default",
            PrefType::Locked => "Locked",
            PrefType::Sticky => "Sticky",
        };

        let value_str = match &entry.value {
            PrefValue::Bool(b) => b.to_string(),
            PrefValue::Integer(n) => n.to_string(),
            PrefValue::Float(f) => format!("{:.2}", f),
            PrefValue::String(s) => {
                if s.len() > 15 {
                    format!("{}...", &s[..15])
                } else {
                    s.clone()
                }
            }
            PrefValue::Null => "null".to_string(),
        };

        println!("{:<40} {:<10} {:<20}", entry.key, type_name, value_str);
    }

    println!("{:-<80}", "");

    // Show some statistics
    let (user_count, default_count, locked_count, sticky_count) = entries.iter().fold(
        (0, 0, 0, 0),
        |(user, default, locked, sticky), entry| match entry.pref_type {
            PrefType::User => (user + 1, default, locked, sticky),
            PrefType::Default => (user, default + 1, locked, sticky),
            PrefType::Locked => (user, default, locked + 1, sticky),
            PrefType::Sticky => (user, default, locked, sticky + 1),
        },
    );

    println!("\nStatistics:");
    println!("  User preferences:      {}", user_count);
    println!("  Default preferences:   {}", default_count);
    println!("  Locked preferences:    {}", locked_count);
    println!("  Sticky preferences:    {}", sticky_count);

    Ok(())
}
