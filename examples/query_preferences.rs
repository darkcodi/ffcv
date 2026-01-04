//! Query preferences example using glob patterns
//!
//! This example demonstrates how to use glob patterns to filter
//! and query Firefox preferences.

use ffcv::parse_prefs_js;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!(
            "Usage: {} <prefs.js path> <pattern1> [pattern2...]",
            args[0]
        );
        eprintln!("\nExamples:");
        eprintln!("  {} /path/to/prefs.js 'network.*'", args[0]);
        eprintln!("  {} /path/to/prefs.js 'browser.*' 'extensions.*'", args[0]);
        eprintln!("  {} /path/to/prefs.js '*.proxy.*'", args[0]);
        return Ok(());
    }

    let path = Path::new(&args[1]);
    let patterns: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();

    // Read the prefs.js file
    println!("Reading preferences from: {}", path.display());
    println!("Query patterns: {:?}\n", patterns);
    let content = fs::read_to_string(path)?;

    // Parse the preferences
    let entries = parse_prefs_js(&content)?;

    println!("Matching preferences:");
    println!("{:-<80}", "");

    // Filter entries using glob pattern matching
    let mut match_count = 0;
    for entry in &entries {
        // Check if any pattern matches this entry
        let matches = patterns
            .iter()
            .any(|pattern| glob_match(pattern, &entry.key));

        if matches {
            match_count += 1;
            println!("{:<50} = {:?}", entry.key, entry.value);
        }
    }

    println!("{:-<80}", "");
    println!(
        "\nTotal matches: {} out of {} preferences",
        match_count,
        entries.len()
    );

    // Show which patterns matched
    println!("\nPattern breakdown:");
    for pattern in &patterns {
        let count = entries
            .iter()
            .filter(|e| glob_match(pattern, &e.key))
            .count();
        println!("  {} - {} matches", pattern, count);
    }

    Ok(())
}

/// Simple glob pattern matching
fn glob_match(pattern: &str, text: &str) -> bool {
    // Convert glob pattern to regex
    // In glob patterns: * matches any characters, ? matches single character
    // Dots are literal dots (not regex wildcards)
    let regex_pattern = pattern.replace('*', ".*").replace('?', ".");

    // Simple matching - check if text matches the pattern
    let parts: Vec<&str> = regex_pattern.split(".*").collect();
    if parts.is_empty() {
        return true;
    }

    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if i == 0 {
            // First part must match at the beginning
            if !text.starts_with(part) {
                return false;
            }
            pos = part.len();
        } else if i == parts.len() - 1 {
            // Last part must match at the end
            if !text[pos..].ends_with(part) {
                return false;
            }
        } else {
            // Middle parts can be anywhere
            if let Some(idx) = text[pos..].find(part) {
                pos += idx + part.len();
            } else {
                return false;
            }
        }
    }

    true
}
