// Example: List Firefox Installations
//
// This example demonstrates how to discover and list Firefox installations
// on your system using ffcv.

use ffcv::{find_all_firefox_installations, find_firefox_installation};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Firefox Installation Discovery");
    println!("=============================\n");

    // Find the primary Firefox installation
    match find_firefox_installation()? {
        Some(installation) => {
            println!("Primary Firefox Installation:");
            println!("  Path: {}", installation.path.display());
            println!("  Version: {}", installation.version);
            println!("  Has omni.ja: {}", installation.has_omni_ja);
            println!("  Has greprefs.js: {}", installation.has_greprefs);
            println!();
        }
        None => println!("No Firefox installation found\n"),
    }

    // Find all Firefox installations
    println!("All Firefox Installations:");
    let installations = find_all_firefox_installations()?;

    if installations.is_empty() {
        println!("  No installations found");
    } else {
        for (i, installation) in installations.iter().enumerate() {
            println!("\n[Installation {}]", i + 1);
            println!("  Path: {}", installation.path.display());
            println!("  Version: {}", installation.version);
            println!("  omni.ja: {}", installation.has_omni_ja);
            println!("  greprefs.js: {}", installation.has_greprefs);
        }
    }

    Ok(())
}
