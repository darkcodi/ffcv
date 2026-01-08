//! Preference merger
//!
//! This module provides functionality to merge Firefox preferences from
//! multiple sources (built-in defaults, global defaults, and user preferences)
//! with proper precedence handling.

use crate::error::{Error, Result};
use crate::firefox_locator;
use crate::omni_extractor::OmniExtractor;
use crate::parser::parse_prefs_js_file;
use crate::types::{MergedPreferences, PrefEntry, PrefSource};
use std::collections::HashMap;
use std::path::Path;

/// Configuration for preference merging
///
/// Controls which sources are included and how errors are handled.
///
/// # Example
///
/// ```rust,no_run
/// use ffcv::MergeConfig;
///
/// let config = MergeConfig {
///     include_builtins: true,
///     include_globals: true,
///     include_user: true,
///     continue_on_error: true,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct MergeConfig {
    /// Include built-in defaults from omni.ja
    pub include_builtins: bool,
    /// Include global defaults from greprefs.js
    pub include_globals: bool,
    /// Include user preferences from prefs.js
    pub include_user: bool,
    /// Continue even if some sources fail to load
    pub continue_on_error: bool,
}

impl Default for MergeConfig {
    fn default() -> Self {
        Self {
            include_builtins: true,
            include_globals: true,
            include_user: true,
            continue_on_error: true,
        }
    }
}

/// Merge preferences from multiple sources
///
/// This function loads preferences from built-in defaults (omni.ja),
/// global defaults (greprefs.js), and user preferences (prefs.js),
/// then merges them with proper precedence:
///
/// 1. Built-in defaults (lowest precedence)
/// 2. Global defaults (medium precedence)
/// 3. User preferences (highest precedence)
///
/// # Arguments
///
/// * `profile_path` - Path to Firefox profile directory
/// * `install_path` - Optional path to Firefox installation (auto-detected if None)
/// * `config` - Merge configuration
///
/// # Returns
///
/// - `Ok(merged)` - Merged preferences with metadata
/// - `Err(_)` - Error during merging (only if continue_on_error is false)
///
/// # Example
///
/// ```rust,no_run
/// use ffcv::{merge_all_preferences, MergeConfig};
/// use std::path::PathBuf;
///
/// let profile_path = PathBuf::from("/home/user/.mozilla/firefox/default");
/// let merged = merge_all_preferences(&profile_path, None, &MergeConfig::default()).unwrap();
///
/// println!("Loaded {} preferences", merged.entries.len());
/// println!("Sources: {:?}", merged.loaded_sources);
/// ```
pub fn merge_all_preferences(
    profile_path: &Path,
    install_path: Option<&Path>,
    config: &MergeConfig,
) -> Result<MergedPreferences> {
    let mut warnings = Vec::new();
    let mut loaded_sources = Vec::new();
    let mut pref_map: HashMap<String, PrefEntry> = HashMap::new();

    // Auto-detect Firefox installation if not provided
    let resolved_install_path = if let Some(path) = install_path {
        Some(path.to_path_buf())
    } else if config.include_builtins || config.include_globals {
        match firefox_locator::find_firefox_installation() {
            Ok(Some(install)) => {
                eprintln!("Found Firefox {} at {:?}", install.version, install.path);
                loaded_sources.push(PrefSource::BuiltIn);
                Some(install.path)
            }
            Ok(None) => {
                warnings.push("Firefox installation not found".to_string());
                None
            }
            Err(e) => {
                warnings.push(format!("Failed to locate Firefox: {}", e));
                None
            }
        }
    } else {
        None
    };

    // Load built-in defaults from omni.ja (lowest precedence)
    if config.include_builtins {
        if let Some(ref install) = resolved_install_path {
            match load_builtin_preferences(install, &mut warnings) {
                Ok(builtins) => {
                    eprintln!(
                        "Loaded {} built-in preferences from omni.ja",
                        builtins.len()
                    );
                    for pref in builtins {
                        pref_map.insert(pref.key.clone(), pref);
                    }
                    loaded_sources.push(PrefSource::BuiltIn);
                }
                Err(e) => {
                    let msg = format!("Failed to load built-in preferences: {}", e);
                    warnings.push(msg.clone());
                    if !config.continue_on_error {
                        return Err(Error::OmniJaError(msg));
                    }
                }
            }
        }
    }

    // Load global defaults from greprefs.js (medium precedence)
    if config.include_globals {
        if let Some(ref install) = resolved_install_path {
            match load_global_preferences(install, &mut warnings) {
                Ok(globals) => {
                    eprintln!(
                        "Loaded {} global preferences from greprefs.js",
                        globals.len()
                    );
                    for pref in globals {
                        pref_map.insert(pref.key.clone(), pref);
                    }
                    loaded_sources.push(PrefSource::GlobalDefault);
                }
                Err(e) => {
                    let msg = format!("Failed to load global preferences: {}", e);
                    warnings.push(msg);
                    if !config.continue_on_error {
                        return Err(Error::PrefFileNotFound {
                            file: "greprefs.js".to_string(),
                        });
                    }
                }
            }
        }
    }

    // Load user preferences from prefs.js (highest precedence)
    if config.include_user {
        let prefs_js_path = profile_path.join("prefs.js");

        match load_user_preferences(&prefs_js_path, &mut warnings) {
            Ok(user_prefs) => {
                eprintln!("Loaded {} user preferences from prefs.js", user_prefs.len());
                for pref in user_prefs {
                    pref_map.insert(pref.key.clone(), pref);
                }
                loaded_sources.push(PrefSource::User);
            }
            Err(e) => {
                let msg = format!("Failed to load user preferences: {}", e);
                warnings.push(msg);
                if !config.continue_on_error {
                    return Err(e);
                }
            }
        }
    }

    // Convert HashMap to Vec
    let mut entries: Vec<PrefEntry> = pref_map.into_values().collect();
    entries.sort_by(|a, b| a.key.cmp(&b.key));

    Ok(MergedPreferences {
        entries,
        install_path: resolved_install_path,
        profile_path: profile_path.to_path_buf(),
        loaded_sources,
        warnings,
    })
}

/// Get the effective value for a preference key
///
/// Returns the highest-precedence preference entry matching the given key.
///
/// # Arguments
///
/// * `prefs` - Slice of preference entries
/// * `key` - Preference key to look up
///
/// # Returns
///
/// - `Some(entry)` - Found preference entry
/// - `None` - Preference not found
///
/// # Example
///
/// ```rust
/// use ffcv::get_effective_pref;
/// use ffcv::{parse_prefs_js, PrefEntry};
///
/// let content = r#"user_pref("test", true);"#;
/// let prefs = parse_prefs_js(content).unwrap();
///
/// if let Some(entry) = get_effective_pref(&prefs, "test") {
///     println!("Found: {:?}", entry.value);
/// }
/// ```
pub fn get_effective_pref<'a>(prefs: &'a [PrefEntry], key: &str) -> Option<&'a PrefEntry> {
    prefs.iter().find(|e| e.key == key)
}

/// Load built-in preferences from omni.ja
fn load_builtin_preferences(
    install_path: &Path,
    warnings: &mut Vec<String>,
) -> Result<Vec<PrefEntry>> {
    // Find omni.ja (try browser/ subdirectory first, then root)
    let omni_paths = [
        install_path.join("browser/omni.ja"),
        install_path.join("omni.ja"),
    ];

    let omni_path = omni_paths.iter().find(|p| p.exists()).ok_or_else(|| {
        warnings.push("omni.ja not found in Firefox installation".to_string());
        Error::PrefFileNotFound {
            file: "omni.ja".to_string(),
        }
    })?;

    let extractor = OmniExtractor::new(omni_path.clone())?;
    let extracted_files = extractor.extract_prefs()?;

    let mut all_prefs = Vec::new();

    for file_path in extracted_files {
        match parse_prefs_js_file(&file_path) {
            Ok(mut prefs) => {
                // Update source information for each preference
                for pref in &mut prefs {
                    pref.source = Some(PrefSource::BuiltIn);
                    if let Ok(file_name) = file_path.strip_prefix(install_path) {
                        pref.source_file = Some(format!("omni.ja:{}", file_name.display()));
                    }
                }
                all_prefs.extend(prefs);
            }
            Err(e) => {
                warnings.push(format!("Failed to parse {}: {}", file_path.display(), e));
            }
        }
    }

    Ok(all_prefs)
}

/// Load global preferences from greprefs.js
fn load_global_preferences(
    install_path: &Path,
    warnings: &mut Vec<String>,
) -> Result<Vec<PrefEntry>> {
    let greprefs_paths = [
        install_path.join("greprefs.js"),
        install_path.join("browser/greprefs.js"),
    ];

    let greprefs_path = greprefs_paths.iter().find(|p| p.exists()).ok_or_else(|| {
        warnings.push("greprefs.js not found in Firefox installation".to_string());
        Error::PrefFileNotFound {
            file: "greprefs.js".to_string(),
        }
    })?;

    let mut prefs = parse_prefs_js_file(greprefs_path)?;

    // Update source information
    for pref in &mut prefs {
        pref.source = Some(PrefSource::GlobalDefault);
        pref.source_file = Some("greprefs.js".to_string());
    }

    Ok(prefs)
}

/// Load user preferences from prefs.js
fn load_user_preferences(
    prefs_js_path: &Path,
    warnings: &mut Vec<String>,
) -> Result<Vec<PrefEntry>> {
    if !prefs_js_path.exists() {
        warnings.push(format!("prefs.js not found at {}", prefs_js_path.display()));
        return Err(Error::PrefFileNotFound {
            file: prefs_js_path.display().to_string(),
        });
    }

    parse_prefs_js_file(prefs_js_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PrefType, PrefValue};
    use std::fs::write;
    use tempfile::TempDir;

    #[test]
    fn test_merge_config_default() {
        let config = MergeConfig::default();
        assert!(config.include_builtins);
        assert!(config.include_globals);
        assert!(config.include_user);
        assert!(config.continue_on_error);
    }

    #[test]
    fn test_get_effective_pref() {
        let prefs = vec![PrefEntry {
            key: "test.pref".to_string(),
            value: PrefValue::Bool(true),
            pref_type: PrefType::User,
            explanation: None,
            source: Some(PrefSource::User),
            source_file: Some("prefs.js".to_string()),
        }];

        assert!(get_effective_pref(&prefs, "test.pref").is_some());
        assert!(get_effective_pref(&prefs, "nonexistent").is_none());
    }

    #[test]
    fn test_load_user_preferences() {
        let temp_dir = TempDir::new().unwrap();
        let prefs_path = temp_dir.path().join("prefs.js");

        let content = r#"
            user_pref("test.pref", true);
            user_pref("another.pref", "value");
        "#;

        write(&prefs_path, content).unwrap();

        let mut warnings = Vec::new();
        let prefs = load_user_preferences(&prefs_path, &mut warnings).unwrap();

        assert_eq!(prefs.len(), 2);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_load_user_preferences_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let prefs_path = temp_dir.path().join("nonexistent.js");

        let mut warnings = Vec::new();
        let result = load_user_preferences(&prefs_path, &mut warnings);

        assert!(result.is_err());
        assert!(!warnings.is_empty());
    }
}
