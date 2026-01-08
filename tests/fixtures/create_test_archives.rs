use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create omni.ja for ESR115
    create_zip(
        "tests/fixtures/firefox-esr115/omni-extracted",
        "tests/fixtures/firefox-esr115/omni-ja-esr115.ja",
    )?;

    // Create omni.ja for Release128
    create_zip(
        "tests/fixtures/firefox-release128/omni-extracted",
        "tests/fixtures/firefox-release128/omni-ja-release128.ja",
    )?;

    println!("Created test omni.ja files successfully");
    Ok(())
}

fn create_zip(source_dir: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use zip::write::FileOptions;
    use zip::ZipWriter;

    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Add all files from source_dir to the zip
    let source_path = Path::new(source_dir);

    // Manually add the files we know exist
    let files = vec![
        "defaults/pref/browser.js",
        "defaults/pref/firefox.js",
    ];

    for file_path in files {
        let full_path = source_path.join(file_path);
        let contents = std::fs::read(&full_path)?;
        zip.start_file(file_path, options)?;
        zip.write_all(&contents)?;
    }

    zip.finish()?;
    println!("Created {}", output_path);
    Ok(())
}
