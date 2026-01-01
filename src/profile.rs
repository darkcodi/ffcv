use anyhow::Context;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Firefox profile information parsed from profiles.ini
#[derive(Debug, Clone)]
struct FirefoxProfile {
    name: String,
    path: PathBuf,
    is_relative: bool,
    is_default: bool,
}

/// Public profile information for listing
#[derive(Debug, Serialize)]
pub struct ProfileInfo {
    pub name: String,
    pub path: PathBuf,
    pub is_default: bool,
    pub is_relative: bool,
    pub locked_to_install: Option<String>,
}

/// Find the Firefox profile directory based on the profile name
pub fn find_profile_path(
    profile_name: &str,
    profiles_dir_opt: Option<&std::path::Path>,
) -> Result<PathBuf, anyhow::Error> {
    let profiles_dir = get_profiles_directory(profiles_dir_opt)?;
    let profiles_ini = profiles_dir.join("profiles.ini");

    // Try parsing profiles.ini first (primary method)
    if profiles_ini.exists() {
        if let Ok(profiles) = parse_profiles_ini(&profiles_ini) {
            // First try exact name match
            if let Some(profile) = profiles.iter().find(|p| p.name == profile_name) {
                let full_path = if profile.is_relative {
                    profiles_dir.join(&profile.path)
                } else {
                    profile.path.clone()
                };

                if full_path.exists() {
                    return Ok(full_path);
                }
            }

            // If no exact match and profile_name is "default", try to find the default
            // profile for the current Firefox installation (Firefox 67+)
            if profile_name == "default" {
                if let Ok(default_profile) = get_default_profile_for_install(
                    &profiles_ini,
                    profiles_dir.as_path(),
                    &profiles,
                ) {
                    return Ok(default_profile);
                }

                // Fallback: use the profile marked as default in profiles.ini
                if let Some(profile) = profiles.iter().find(|p| p.is_default) {
                    let full_path = if profile.is_relative {
                        profiles_dir.join(&profile.path)
                    } else {
                        profile.path.clone()
                    };

                    if full_path.exists() {
                        return Ok(full_path);
                    }
                }
            }
        }
    }

    // Fallback: Directory scanning with improved matching
    scan_profiles_directory(&profiles_dir, profile_name)
}

/// Parse profiles.ini to extract profile information
fn parse_profiles_ini(ini_path: &PathBuf) -> Result<Vec<FirefoxProfile>, anyhow::Error> {
    use configparser::ini::Ini;

    let mut ini = Ini::new();
    let content = std::fs::read_to_string(ini_path)
        .with_context(|| format!("Failed to read profiles.ini from {}", ini_path.display()))?;

    // Parse the INI file (configparser handles UTF-8 BOM automatically)
    if let Err(e) = ini.read(content) {
        return Err(anyhow::anyhow!("Failed to parse profiles.ini: {}", e));
    }

    let mut profiles = Vec::new();

    // Get all section names
    let sections = ini.sections();

    for sec_name in sections {
        // Only process profilen sections (e.g., profile0, profile1, etc.)
        // Note: configparser converts section names to lowercase
        if sec_name.to_lowercase().starts_with("profile") {
            let name = ini.get(&sec_name, "Name").unwrap_or_default();
            let path_str = ini.get(&sec_name, "Path").unwrap_or_default();
            let is_relative = ini
                .getuint(&sec_name, "IsRelative")
                .ok()
                .flatten()
                .unwrap_or(1)
                == 1;
            let is_default = ini
                .getuint(&sec_name, "Default")
                .ok()
                .flatten()
                .unwrap_or(0)
                == 1;

            if !name.is_empty() && !path_str.is_empty() {
                profiles.push(FirefoxProfile {
                    name,
                    path: PathBuf::from(path_str),
                    is_relative,
                    is_default,
                });
            }
        }
    }

    Ok(profiles)
}

/// Parse install sections from profiles.ini (Firefox 67+)
/// Returns HashMap<install_hash, default_profile_path>
fn parse_installs_ini(ini_path: &PathBuf) -> Result<HashMap<String, String>, anyhow::Error> {
    use configparser::ini::Ini;

    let mut ini = Ini::new();
    let content = std::fs::read_to_string(ini_path)?;

    if let Err(e) = ini.read(content) {
        return Err(anyhow::anyhow!("Failed to parse profiles.ini: {}", e));
    }

    let mut installs = HashMap::new();

    // Get all section names
    let sections = ini.sections();

    for sec_name in sections {
        // Install sections are named with hash (e.g., [308046B0AF4A39CB])
        // They don't start with "profile" or "general"
        // Note: configparser converts section names to lowercase
        let sec_lower = sec_name.to_lowercase();
        if !sec_lower.starts_with("profile") && sec_lower != "general" {
            if let Some(default_profile) = ini.get(&sec_name, "Default") {
                installs.insert(sec_name, default_profile);
            }
        }
    }

    Ok(installs)
}

/// Get the default profile for the current Firefox installation (Firefox 67+)
fn get_default_profile_for_install(
    ini_path: &PathBuf,
    profiles_dir: &Path,
    profiles: &[FirefoxProfile],
) -> Result<PathBuf, anyhow::Error> {
    use configparser::ini::Ini;

    // Try to find the Firefox installation path
    let firefox_path = get_firefox_install_path()?;

    // Hash the installation path to get the install section name
    // Firefox uses a simple hash of the installation path
    let install_hash = hash_install_path(&firefox_path);

    // Parse profiles.ini to get the install mapping
    let mut ini = Ini::new();
    let content = std::fs::read_to_string(ini_path)?;

    if let Err(e) = ini.read(content) {
        return Err(anyhow::anyhow!("Failed to parse profiles.ini: {}", e));
    }

    // Look for the install section with the matching hash
    let sections = ini.sections();
    for sec_name in sections {
        let sec_lower = sec_name.to_lowercase();
        // Check if this is an install section and if the hash matches
        if !sec_lower.starts_with("profile")
            && sec_lower != "general"
            && sec_lower == install_hash.to_lowercase()
        {
            // Found the install section! Get the default profile
            if let Some(default_profile_path) = ini.get(&sec_name, "Default") {
                // Find the profile with this path
                if let Some(profile) = profiles.iter().find(|p| {
                    p.path.to_string_lossy() == default_profile_path
                        || p.path.to_string_lossy() == format!("Profiles/{}", default_profile_path)
                }) {
                    let full_path = if profile.is_relative {
                        profiles_dir.join(&profile.path)
                    } else {
                        profile.path.clone()
                    };

                    if full_path.exists() {
                        return Ok(full_path);
                    }
                }

                // If profile not found in list, try to construct path directly
                let profile_path = PathBuf::from(&default_profile_path);
                let full_path = if profile_path.is_absolute() {
                    profile_path
                } else {
                    profiles_dir.join(&profile_path)
                };

                if full_path.exists() {
                    return Ok(full_path);
                }
            }
        }
    }

    // No install section found for this installation
    Err(anyhow::anyhow!(
        "No default profile found for Firefox installation at {}",
        firefox_path.display()
    ))
}

/// Get the path to the Firefox executable
fn get_firefox_install_path() -> Result<PathBuf, anyhow::Error> {
    use std::env;

    // Try to find Firefox in PATH
    if let Ok(path) = env::var("FIREFOX_BIN") {
        return Ok(PathBuf::from(path));
    }

    // Try common locations
    #[cfg(target_os = "linux")]
    {
        let common_paths = [
            "/usr/bin/firefox",
            "/usr/lib/firefox/firefox",
            "/snap/bin/firefox",
            "/opt/firefox/firefox",
        ];

        for path in &common_paths {
            if PathBuf::from(path).exists() {
                return Ok(PathBuf::from(path));
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let common_paths = [
            "/Applications/Firefox.app/Contents/MacOS/firefox",
            "/Applications/Firefox Developer Edition.app/Contents/MacOS/firefox",
        ];

        for path in &common_paths {
            if PathBuf::from(path).exists() {
                return Ok(PathBuf::from(path));
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(program_files) = env::var("PROGRAMFILES") {
            let common_paths = [
                format!("{}/Mozilla Firefox/firefox.exe", program_files),
                format!("{}\\Mozilla Firefox\\firefox.exe", program_files),
            ];

            for path in &common_paths {
                if PathBuf::from(path).exists() {
                    return Ok(PathBuf::from(path));
                }
            }
        }
    }

    // Fallback: try to use 'which' or 'where' command
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("which").arg("firefox").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                let path = path_str.trim();
                if !path.is_empty() {
                    return Ok(PathBuf::from(path));
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("where")
            .arg("firefox.exe")
            .output()
        {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                let path = path_str.lines().next().unwrap_or("").trim();
                if !path.is_empty() {
                    return Ok(PathBuf::from(path));
                }
            }
        }
    }

    Err(anyhow::anyhow!("Could not find Firefox installation"))
}

/// Hash the installation path to get the install section name
/// Firefox uses CityHash64 to hash the installation directory path
fn hash_install_path(path: &Path) -> String {
    use cityhasher::hash;

    // Resolve symlinks first (critical for NixOS and similar systems)
    let resolved = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    // Get parent directory (installation directory)
    // Firefox hashes the installation directory, not the binary path
    let install_dir = resolved.parent().unwrap_or(&resolved);

    // Normalize path string
    let path_str = install_dir.to_string_lossy();

    // Hash with CityHash64 and format as 16 uppercase hex chars
    let hash: u64 = hash(path_str.as_bytes());
    format!("{:016X}", hash)
}

/// Improved fallback: Scan profiles directory with better matching strategies
fn scan_profiles_directory(
    profiles_dir: &PathBuf,
    profile_name: &str,
) -> Result<PathBuf, anyhow::Error> {
    let entries = std::fs::read_dir(profiles_dir).with_context(|| {
        format!(
            "Failed to read profiles directory: {}",
            profiles_dir.display()
        )
    })?;

    let mut matches: Vec<PathBuf> = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

            // Strategy 1: Exact match (uncommon but possible)
            if dir_name == profile_name {
                return Ok(path);
            }

            // Strategy 2: Firefox standard naming pattern (xxxxxxxx.name)
            if dir_name.ends_with(&format!(".{}", profile_name)) {
                matches.push(path);
            }
        }
    }

    // If exactly one match, use it
    if matches.len() == 1 {
        return Ok(matches.into_iter().next().unwrap());
    }

    // If multiple matches, return error listing them
    if matches.len() > 1 {
        let match_names: Vec<&str> = matches
            .iter()
            .filter_map(|p| p.file_name().and_then(|s| s.to_str()))
            .collect();

        return Err(anyhow::anyhow!(
            "Multiple profiles match '{}': {}. \
             Please specify the exact profile name from 'ffcv --list' \
             or use the full directory name.",
            profile_name,
            match_names.join(", ")
        ));
    }

    // No matches
    Err(anyhow::anyhow!(
        "Profile '{}' not found in {}. \
         Use 'ffcv --list' to see available profiles.",
        profile_name,
        profiles_dir.display()
    ))
}

/// List all available Firefox profiles
pub fn list_profiles(
    profiles_dir_opt: Option<&std::path::Path>,
) -> Result<Vec<ProfileInfo>, anyhow::Error> {
    let profiles_dir = get_profiles_directory(profiles_dir_opt)?;
    let profiles_ini = profiles_dir.join("profiles.ini");

    if !profiles_ini.exists() {
        return Err(anyhow::anyhow!(
            "profiles.ini not found at {}. \
             Firefox may not be installed or this is not a standard Firefox setup.",
            profiles_ini.display()
        ));
    }

    let profiles = parse_profiles_ini(&profiles_ini)?;
    let installs = parse_installs_ini(&profiles_ini)?;

    // Merge profile info with install locks
    let profile_infos: Vec<ProfileInfo> = profiles
        .into_iter()
        .map(|p| {
            let path_string = p.path.to_string_lossy().to_string();
            let locked_to = installs
                .iter()
                .find(|(_, default_path)| *default_path == &path_string)
                .map(|(hash, _)| hash.clone());

            ProfileInfo {
                name: p.name,
                path: p.path,
                is_default: p.is_default,
                is_relative: p.is_relative,
                locked_to_install: locked_to,
            }
        })
        .collect();

    Ok(profile_infos)
}

/// Get the profiles directory path from CLI, env var, or auto-detection
///
/// Priority:
/// 1. Manual path provided via CLI or parameter
/// 2. MOZ_PROFILES_DIR environment variable
/// 3. Auto-detection based on OS
pub fn get_profiles_directory(
    manual_path: Option<&std::path::Path>,
) -> Result<PathBuf, anyhow::Error> {
    // Priority 1: Use manually specified path (from CLI or direct parameter)
    if let Some(path) = manual_path {
        return validate_and_use_profiles_dir(path);
    }

    // Priority 2: Check MOZ_PROFILES_DIR environment variable
    if let Ok(env_path) = std::env::var("MOZ_PROFILES_DIR") {
        let path = PathBuf::from(env_path);
        return validate_and_use_profiles_dir(&path);
    }

    // Priority 3: Auto-detect based on OS
    auto_detect_profiles_directory()
}

/// Validate and return the profiles directory path
fn validate_and_use_profiles_dir(path: &std::path::Path) -> Result<PathBuf, anyhow::Error> {
    // Check if path exists
    if !path.exists() {
        return Err(anyhow::anyhow!(
            "Profiles directory does not exist: {}\n\
             Please verify the path and try again.",
            path.display()
        ));
    }

    // Check if path is a directory
    if !path.is_dir() {
        return Err(anyhow::anyhow!(
            "Profiles directory path is not a directory: {}\n\
             Please provide a directory path, not a file.",
            path.display()
        ));
    }

    Ok(path.to_path_buf())
}

/// Auto-detect profiles directory based on operating system
fn auto_detect_profiles_directory() -> Result<PathBuf, anyhow::Error> {
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
        Ok(PathBuf::from(home).join("Library/Application Support/Firefox"))
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| anyhow::anyhow!("APPDATA environment variable not set"))?;
        Ok(PathBuf::from(appdata).join("Mozilla/Firefox"))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(anyhow::anyhow!("Unsupported operating system"))
    }
}

/// Get the path to prefs.js for a given profile
pub fn get_prefs_path(profile_path: &std::path::Path) -> PathBuf {
    profile_path.join("prefs.js")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_prefs_path() {
        let profile_path = PathBuf::from("/home/user/.mozilla/firefox/test.default");
        let prefs_path = get_prefs_path(&profile_path);
        assert_eq!(
            prefs_path,
            PathBuf::from("/home/user/.mozilla/firefox/test.default/prefs.js")
        );
    }

    #[test]
    fn test_parse_valid_profiles_ini() {
        let ini_content = r#"
[General]
StartWithLastProfile=1
Version=2

[Profile0]
Name=default
IsRelative=1
Path=Profiles/abcdefgh.default
Default=1

[Profile1]
Name=work
IsRelative=1
Path=Profiles/work.profile
Default=0
"#;

        let mut ini = configparser::ini::Ini::new();
        ini.read(ini_content.to_string()).unwrap();

        // Verify General section
        assert_eq!(
            ini.get("General", "StartWithLastProfile"),
            Some("1".to_string())
        );

        // Verify Profile0 section
        assert_eq!(ini.get("Profile0", "Name"), Some("default".to_string()));
        assert_eq!(
            ini.get("Profile0", "Path"),
            Some("Profiles/abcdefgh.default".to_string())
        );
    }

    #[test]
    fn test_nixos_profile_name_mismatch() {
        // Test the scenario where profile name doesn't match directory name
        // This happens on NixOS with home-manager
        let ini_content = r#"
[General]
StartWithLastProfile=1

[Profile0]
Name=darkcodi
IsRelative=1
Path=default
Default=1
"#;

        let mut ini = configparser::ini::Ini::new();
        ini.read(ini_content.to_string()).unwrap();

        assert_eq!(ini.get("Profile0", "Name"), Some("darkcodi".to_string()));
        assert_eq!(ini.get("Profile0", "Path"), Some("default".to_string()));

        // Verify IsRelative is parsed correctly
        let is_relative = ini
            .get("Profile0", "IsRelative")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1);
        assert_eq!(is_relative, 1);
    }

    #[test]
    fn test_parse_installs_section() {
        // Test Firefox 67+ install section parsing
        let ini_content = r#"
[General]
StartWithLastProfile=1

[Profile0]
Name=default
IsRelative=1
Path=Profiles/abcdefgh.default
Default=1

[308046B0AF4A39CB]
Default=Profiles/abcdefgh.default
Locked=1
"#;

        let mut ini = configparser::ini::Ini::new();
        ini.read(ini_content.to_string()).unwrap();

        // Verify install section (hash based section name)
        assert_eq!(
            ini.get("308046B0AF4A39CB", "Default"),
            Some("Profiles/abcdefgh.default".to_string())
        );
        assert_eq!(ini.get("308046B0AF4A39CB", "Locked"), Some("1".to_string()));
    }

    #[test]
    fn test_profiles_dir_validation_nonexistent() {
        // Test validation with non-existent path
        let result = get_profiles_directory(Some(std::path::Path::new("/nonexistent/path")));
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("does not exist"));
        assert!(error_msg.contains("/nonexistent/path"));
    }

    #[test]
    fn test_profiles_dir_validation_file_not_directory() {
        // Test validation when path is a file, not a directory
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let result = get_profiles_directory(Some(temp_file.path()));
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("not a directory"));
    }

    #[test]
    fn test_profiles_dir_validation_valid_directory() {
        // Test validation with a valid temporary directory
        let temp_dir = tempfile::TempDir::new().unwrap();
        let result = get_profiles_directory(Some(temp_dir.path()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path());
    }
}
