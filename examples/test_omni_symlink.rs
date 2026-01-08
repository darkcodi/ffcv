use std::fs::File;
use std::path::Path;

fn main() {
    // Test the symlink path
    let symlink_path = Path::new(
        "/nix/store/7r9qm8f1fdpg762rv45594qlnmvjxk87-firefox-146.0.1/lib/firefox/omni.ja",
    );
    println!("Testing symlink path: {:?}", symlink_path);
    println!("Symlink exists: {}", symlink_path.exists());
    println!("Symlink is_symlink: {}", symlink_path.is_symlink());

    match File::open(symlink_path) {
        Ok(file) => {
            println!(
                "File opened successfully, size: {:?}",
                file.metadata().map(|m| m.len())
            );
            match zip::ZipArchive::new(file) {
                Ok(archive) => println!("✓ Zip archive opened! Files: {}", archive.len()),
                Err(e) => println!("✗ Zip error: {}", e),
            }
        }
        Err(e) => println!("✗ Open error: {}", e),
    }
}
