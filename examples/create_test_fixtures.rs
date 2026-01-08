use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process;
use zip::write::FileOptions;
use zip::ZipWriter;

/// Helper program to create test omni.ja archives for testing
/// Usage: cargo run --example create_test_fixtures
fn main() {
    if let Err(e) = create_test_archives() {
        eprintln!("Error creating test archives: {}", e);
        process::exit(1);
    }
}

fn create_test_archives() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating test omni.ja archives...");

    // Create omni.ja for ESR115
    create_zip_archive(
        "tests/fixtures/firefox-esr115/omni-extracted",
        "tests/fixtures/firefox-esr115/omni-ja-esr115.ja",
    )?;

    // Create omni.ja for Release128
    create_zip_archive(
        "tests/fixtures/firefox-release128/omni-extracted",
        "tests/fixtures/firefox-release128/omni-ja-release128.ja",
    )?;

    println!("\n✅ Successfully created test omni.ja files:");
    println!("   - tests/fixtures/firefox-esr115/omni-ja-esr115.ja");
    println!("   - tests/fixtures/firefox-release128/omni-ja-release128.ja");

    Ok(())
}

fn create_zip_archive(
    source_dir: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let source_path = Path::new(source_dir);

    // Add all files from source_dir to the zip
    let files = vec!["defaults/pref/browser.js", "defaults/pref/firefox.js"];

    println!("Creating {}...", output_path);

    for file_path in files {
        let full_path = source_path.join(file_path);
        if full_path.exists() {
            let contents = std::fs::read(&full_path)?;
            zip.start_file(file_path, options)?;
            zip.write_all(&contents)?;
            println!("  Added: {}", file_path);
        } else {
            eprintln!("  Warning: {} not found, skipping", file_path);
        }
    }

    zip.finish()?;
    Ok(())
}
