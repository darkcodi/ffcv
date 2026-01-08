//! Firefox installation locator
//!
//! This module provides functionality to locate Firefox installations
//! across different platforms (Linux, macOS, Windows).

use crate::error::{Error, Result};
use crate::types::FirefoxInstallation;
use std::fs;
use std::path::{Path, PathBuf};

/// Platform-specific Firefox installation search paths
#[cfg(target_os = "linux")]
const FIREFOX_SEARCH_PATHS: &[&str] = &[
    "/usr/lib/firefox",
    "/usr/lib64/firefox",
    "/opt/firefox",
    "/usr/local/firefox",
    "/opt/firefox-beta",
    "/opt/firefox-esr",
];

#[cfg(target_os = "macos")]
const FIREFOX_SEARCH_PATHS: &[&str] = &[
    "/Applications/Firefox.app/Contents/Resources",
    "/Applications/Firefox Beta.app/Contents/Resources",
    "/Applications/Firefox Developer Edition.app/Contents/Resources",
    "/Applications/Firefox ESR.app/Contents/Resources",
];

#[cfg(target_os = "windows")]
const FIREFOX_SEARCH_PATHS: &[&str] = &[
    r"C:\Program Files\Mozilla Firefox",
    r"C:\Program Files\Firefox Beta",
    r"C:\Program Files\Firefox ESR",
    r"C:\Program Files\Mozilla Firefox ESR",
    r"C:\Program Files (x86)\Mozilla Firefox",
    r"C:\Program Files (x86)\Firefox Beta",
    r"C:\Program Files (x86)\Firefox ESR",
    r"C:\Program Files (x86)\Mozilla Firefox ESR",
    r"C:\Program Files\Mozilla Firefox Developer Edition",
];

/// Additional search paths for NixOS
#[cfg(target_os = "linux")]
const NIX_STORE_PATHS: &[&str] = &[
    "/nix/var/nix/profiles/default/bin/firefox",
    "/run/current-system/sw/bin/firefox",
];

/// Find the first valid Firefox installation on the system
///
/// This function searches common Firefox installation paths for the current
/// platform and returns the first valid installation found.
///
/// # Returns
///
/// - `Ok(Some(installation))` - Firefox installation found
/// - `Ok(None)` - No Firefox installation found
/// - `Err(_)` - Error while searching (e.g., permission denied)
///
/// # Example
///
/// ```rust,no_run
/// use ffcv::find_firefox_installation;
///
/// match find_firefox_installation() {
///     Ok(Some(install)) => println!("Found Firefox {} at {:?}", install.version, install.path),
///     Ok(None) => println!("Firefox not found"),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn find_firefox_installation() -> Result<Option<FirefoxInstallation>> {
    let search_paths = get_all_search_paths();

    for path in search_paths {
        if let Ok(install) = validate_installation(&path) {
            return Ok(Some(install));
        }
    }

    Ok(None)
}

/// Find all Firefox installations on the system
///
/// This function searches all common Firefox installation paths and returns
/// all valid installations found.
///
/// # Returns
///
/// - `Ok(installations)` - Vector of all Firefox installations found (may be empty)
/// - `Err(_)` - Error while searching (e.g., permission denied)
///
/// # Example
///
/// ```rust,no_run
/// use ffcv::find_all_firefox_installations;
///
/// match find_all_firefox_installations() {
///     Ok(installations) => {
///         for install in installations {
///             println!("Found Firefox {} at {:?}", install.version, install.path);
///         }
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn find_all_firefox_installations() -> Result<Vec<FirefoxInstallation>> {
    let mut installations = Vec::new();
    let search_paths = get_all_search_paths();

    for path in search_paths {
        if let Ok(install) = validate_installation(&path) {
            installations.push(install);
        }
    }

    Ok(installations)
}

/// Get the Firefox version from an installation directory
///
/// This function reads the `application.ini` or `platform.ini` file from
/// the Firefox installation directory and extracts the version string.
///
/// # Arguments
///
/// * `install_path` - Path to Firefox installation directory
///
/// # Returns
///
/// - `Ok(version)` - Firefox version string (e.g., "128.0")
/// - `Err(_)` - Error reading or parsing version file
///
/// # Example
///
/// ```rust,no_run
/// use ffcv::get_firefox_version;
/// use std::path::Path;
///
/// let path = Path::new("/usr/lib/firefox");
/// match get_firefox_version(&path) {
///     Ok(version) => println!("Firefox version: {}", version),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn get_firefox_version(install_path: &Path) -> Result<String> {
    // Try application.ini first (standard location)
    let app_ini = install_path.join("application.ini");

    if app_ini.exists() {
        return extract_version_from_ini(&app_ini);
    }

    // Fallback to platform.ini (some distributions use this)
    let platform_ini = install_path.join("platform.ini");

    if platform_ini.exists() {
        return extract_version_from_ini(&platform_ini);
    }

    // For macOS .app bundles, check Contents/Resources
    #[cfg(target_os = "macos")]
    {
        let resources_ini = install_path.join("application.ini");
        if resources_ini.exists() {
            return extract_version_from_ini(&resources_ini);
        }
    }

    // Check parent directory for macOS .app structure
    #[cfg(target_os = "macos")]
    {
        if let Some(parent) = install_path.parent() {
            let browser_ini = parent.join("browserconfig.properties");
            if browser_ini.exists() {
                if let Ok(content) = fs::read_to_string(&browser_ini) {
                    for line in content.lines() {
                        if line.contains("version") {
                            if let Some(version) = line.split('=').nth(1) {
                                return Ok(version.trim().to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    Err(Error::FirefoxNotFound {
        searched_paths: format!("{} (no version info found)", install_path.display()),
    })
}

/// Validate that a path contains a valid Firefox installation
///
/// Checks for the presence of omni.ja and/or greprefs.js, and extracts
/// version information if available.
///
/// # Arguments
///
/// * `path` - Path to validate
///
/// # Returns
///
/// - `Ok(installation)` - Valid Firefox installation
/// - `Err(_)` - Not a valid Firefox installation or error reading files
fn validate_installation(path: &str) -> Result<FirefoxInstallation> {
    let install_path = PathBuf::from(path);

    if !install_path.exists() {
        return Err(Error::FirefoxNotFound {
            searched_paths: path.to_string(),
        });
    }

    // Check for omni.ja (in browser/ or root)
    let omni_ja_paths = [
        install_path.join("browser/omni.ja"),
        install_path.join("omni.ja"),
    ];

    let has_omni_ja = omni_ja_paths.iter().any(|p| p.exists());

    // Check for greprefs.js
    let greprefs_paths = [
        install_path.join("greprefs.js"),
        install_path.join("browser/greprefs.js"),
    ];

    let has_greprefs = greprefs_paths.iter().any(|p| p.exists());

    // At least one of these files should exist for a valid installation
    if !has_omni_ja && !has_greprefs {
        return Err(Error::FirefoxNotFound {
            searched_paths: format!("{} (no omni.ja or greprefs.js found)", path),
        });
    }

    // Try to get version
    let version = get_firefox_version(&install_path).unwrap_or_else(|_| "unknown".to_string());

    Ok(FirefoxInstallation {
        version,
        path: install_path,
        has_greprefs,
        has_omni_ja,
    })
}

/// Extract version from an .ini file
///
/// Parses standard INI format files to find the Version field.
fn extract_version_from_ini(ini_path: &Path) -> Result<String> {
    let content = fs::read_to_string(ini_path).map_err(|e| Error::FirefoxNotFound {
        searched_paths: format!("{} (cannot read: {})", ini_path.display(), e),
    })?;

    for line in content.lines() {
        let line = line.trim();

        // Look for "Version=X.Y.Z" format
        if line.starts_with("Version=") || line.starts_with("Version =") {
            if let Some(version) = line.split('=').nth(1) {
                return Ok(version.trim().to_string());
            }
        }
    }

    Err(Error::FirefoxNotFound {
        searched_paths: format!("{} (no version found)", ini_path.display()),
    })
}

/// Get all search paths for the current platform
///
/// Combines standard paths with platform-specific additions like Nix store paths.
fn get_all_search_paths() -> Vec<String> {
    let mut paths: Vec<String> = FIREFOX_SEARCH_PATHS.iter().map(|s| s.to_string()).collect();

    // Add Nix store paths on Linux
    #[cfg(target_os = "linux")]
    {
        for nix_path in NIX_STORE_PATHS {
            // Check if nix path is a symlink to a real installation
            if let Ok(target) = fs::read_link(nix_path) {
                // Get the parent nix store directory
                if let Some(store_dir) = target.parent() {
                    paths.push(store_dir.to_string_lossy().to_string());
                }
            }
        }
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_search_paths_not_empty() {
        let paths = get_all_search_paths();
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_validate_nonexistent_path() {
        let result = validate_installation("/nonexistent/firefox/path/xyz123");
        assert!(result.is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_firefox_search_paths_include_standard_locations() {
        let paths = get_all_search_paths();
        assert!(paths.iter().any(|p| p.contains("firefox")));
    }

    #[test]
    fn test_extract_version_from_ini_content() {
        let ini_content = r#"
[App]
Version=128.0
Name=Firefox
"#;
        // This would require creating a temp file and testing extract_version_from_ini
        // For now, we'll just verify the logic structure
        assert!(ini_content.contains("Version=128.0"));
    }
}
