use anyhow::Context;
use std::path::PathBuf;

/// Find the Firefox profile directory based on the profile name and OS
pub fn find_profile_path(profile_name: &str) -> Result<PathBuf, anyhow::Error> {
    let profiles_dir = get_profiles_directory()?;

    // List all directories in the profiles folder
    let entries = std::fs::read_dir(&profiles_dir).with_context(|| {
        format!(
            "Failed to read profiles directory: {}",
            profiles_dir.display()
        )
    })?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

            // Check if this directory matches the profile name
            // Profile directories typically end with the profile name
            if dir_name.contains(profile_name) {
                return Ok(path);
            }
        }
    }

    Err(anyhow::anyhow!(
        "Profile '{}' not found in {}",
        profile_name,
        profiles_dir.display()
    ))
}

/// Get the base profiles directory for the current OS
fn get_profiles_directory() -> Result<PathBuf, anyhow::Error> {
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME")
            .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
        Ok(PathBuf::from(home).join(".mozilla/firefox"))
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME")
            .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
        Ok(PathBuf::from(home).join("Library/Application Support/Firefox/Profiles"))
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| anyhow::anyhow!("APPDATA environment variable not set"))?;
        Ok(PathBuf::from(appdata).join("Mozilla/Firefox/Profiles"))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(anyhow::anyhow!("Unsupported operating system"))
    }
}

/// Get the path to prefs.js for a given profile
pub fn get_prefs_path(profile_path: &PathBuf) -> PathBuf {
    profile_path.join("prefs.js")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_profiles_directory_linux() {
        #[cfg(target_os = "linux")]
        {
            std::env::set_var("HOME", "/home/testuser");
            let dir = get_profiles_directory().unwrap();
            assert!(dir
                .to_string_lossy()
                .contains("/home/testuser/.mozilla/firefox"));
        }
    }

    #[test]
    fn test_get_profiles_directory_macos() {
        #[cfg(target_os = "macos")]
        {
            std::env::set_var("HOME", "/Users/testuser");
            let dir = get_profiles_directory().unwrap();
            assert!(dir
                .to_string_lossy()
                .contains("/Users/testuser/Library/Application Support/Firefox/Profiles"));
        }
    }

    #[test]
    fn test_get_prefs_path() {
        let profile_path = PathBuf::from("/home/user/.mozilla/firefox/Profiles/test.default");
        let prefs_path = get_prefs_path(&profile_path);
        assert_eq!(
            prefs_path,
            PathBuf::from("/home/user/.mozilla/firefox/Profiles/test.default/prefs.js")
        );
    }
}
