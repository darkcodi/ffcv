//! Firefox profile discovery example
//!
//! This example demonstrates how to find and list all Firefox profiles
//! on the current system.

use ffcv::{list_profiles, ProfileInfo};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Firefox Profile Discovery");
    println!("{:-<80}", "");

    // Find all Firefox profiles
    let profiles = list_profiles(None)?;

    if profiles.is_empty() {
        println!("No Firefox profiles found on this system.");
        return Ok(());
    }

    println!("\nFound {} Firefox profile(s):\n", profiles.len());

    for (index, profile) in profiles.iter().enumerate() {
        display_profile(index + 1, profile);
    }

    // Show the default profile
    let default_profiles: Vec<&ProfileInfo> = profiles.iter().filter(|p| p.is_default).collect();

    if !default_profiles.is_empty() {
        println!("{:-<80}", "");
        println!("Default Profile(s):");
        for profile in default_profiles {
            println!("  {} -> {}", profile.name, profile.path.display());
        }
    }

    Ok(())
}

fn display_profile(index: usize, profile: &ProfileInfo) {
    println!("Profile #{}", index);
    println!("  Name:          {}", profile.name);
    println!("  Path:          {}", profile.path.display());
    println!("  Is Default:    {}", profile.is_default);
    println!("  Relative:      {}", profile.is_relative);

    if let Some(ref locked) = profile.locked_to_install {
        println!("  Locked To:      {}", locked);
    }

    // Check if prefs.js exists
    let prefs_path = profile.path.join("prefs.js");
    let prefs_exists = prefs_path.exists();
    println!(
        "  prefs.js:      {}",
        if prefs_exists {
            "✓ Found"
        } else {
            "✗ Not found"
        }
    );

    if prefs_exists {
        // Get file size
        if let Ok(metadata) = fs::metadata(&prefs_path) {
            let size_kb = metadata.len() / 1024;
            println!("  File Size:     {} KB", size_kb);
        }
    }

    println!();
}
