//! # ffcv - Firefox Configuration Viewer Library
//!
//! This library provides functionality for parsing and querying Firefox
//! preference files (prefs.js). It can parse Firefox's custom JavaScript-like
//! preference format, manage Firefox profiles, and filter preferences by glob
//! patterns.
//!
//! ## Features
//!
//! - Parse Firefox prefs.js files with full JavaScript escape sequence support
//! - Extract and merge Firefox's built-in default preferences from omni.ja archives
//! - Auto-discover Firefox installations across platforms (Linux, macOS, Windows)
//! - Detect and manage Firefox profiles across platforms
//! - Query preferences using glob patterns (e.g., `"network.*"`, `"browser.*.enabled"`)
//! - Get human-readable explanations for documented preferences
//! - Support for all four preference types: user, default, locked, and sticky
//! - Track preference sources (built-in, global defaults, user-modified)
//!
//! ## Quick Start
//!
//! ### Parsing Preferences
//!
//! ```rust
//! use ffcv::{parse_prefs_js, PrefType, PrefValue};
//!
//! let content = r#"
//!     user_pref("browser.startup.homepage", "https://example.com");
//!     pref("javascript.enabled", true);
//!     lock_pref("security.default_ssl_enabled", false);
//! "#;
//!
//! let prefs = parse_prefs_js(content)?;
//! let homepage = prefs.iter()
//!     .find(|e| e.key == "browser.startup.homepage")
//!     .unwrap();
//! assert_eq!(homepage.value, PrefValue::String("https://example.com".to_string()));
//! assert_eq!(homepage.pref_type, PrefType::User);
//! # Ok::<(), ffcv::Error>(())
//! ```
//!
//! For convenient value type checking, import the `PrefValueExt` trait:
//!
//! ```rust
//! use ffcv::{parse_prefs_js, PrefValue, PrefValueExt};
//!
//! let content = r#"user_pref("javascript.enabled", true);"#;
//! let prefs = parse_prefs_js(content)?;
//! let entry = &prefs[0];
//!
//! // Use convenience methods from PrefValueExt
//! if let Some(enabled) = entry.value.as_bool() {
//!     println!("JavaScript enabled: {}", enabled);
//! }
//! # Ok::<(), ffcv::Error>(())
//! ```
//!
//! ### Working with Profiles
//!
//! ```rust,no_run
//! use ffcv::{find_profile_path, parse_prefs_js};
//!
//! // Find a specific Firefox profile
//! let profile_path = find_profile_path("default", None)?;
//! let prefs_path = profile_path.join("prefs.js");
//!
//! // Read and parse preferences
//! let content = std::fs::read_to_string(&prefs_path)?;
//! let prefs = parse_prefs_js(&content)?;
//! # Ok::<(), ffcv::Error>(())
//! ```
//!
//! ### Listing All Profiles
//!
//! ```rust,no_run
//! use ffcv::list_profiles;
//!
//! let profiles = list_profiles(None)?;
//! for profile in profiles {
//!     println!("Profile: {} ({})", profile.name, profile.path.display());
//!     if profile.is_default {
//!         println!("  (default profile)");
//!     }
//! }
//! # Ok::<(), ffcv::Error>(())
//! ```
//!
//! ### Querying Preferences
//!
//! ```rust
//! use ffcv::{parse_prefs_js, query_preferences};
//!
//! let content = r#"
//!     user_pref("network.proxy.http", "proxy.example.com");
//!     user_pref("network.proxy.http_port", 8080);
//!     user_pref("browser.startup.homepage", "https://example.com");
//! "#;
//!
//! let prefs = parse_prefs_js(content)?;
//! let network_prefs = query_preferences(&prefs, &["network.*"])?;
//!
//! assert_eq!(network_prefs.len(), 2);
//! assert!(network_prefs.iter().any(|e| e.key == "network.proxy.http"));
//! # Ok::<(), ffcv::Error>(())
//! ```
//!
//! ### Firefox Installation Discovery
//!
//! ```rust,no_run
//! use ffcv::find_firefox_installation;
//!
//! // Auto-detect Firefox installation on your system
//! match find_firefox_installation()? {
//!     Some(installation) => {
//!         println!("Firefox found at: {}", installation.path.display());
//!         println!("Version: {}", installation.version);
//!         println!("Has omni.ja: {}", installation.has_omni_ja);
//!     }
//!     None => println!("No Firefox installation found"),
//! }
//! # Ok::<(), ffcv::Error>(())
//! ```
//!
//! ### Merging All Preference Sources
//!
//! ```rust,no_run
//! use ffcv::{merge_all_preferences, find_profile_path, MergeConfig};
//!
//! let profile_path = find_profile_path("default-release", None)?;
//!
//! let config = MergeConfig {
//!     include_builtins: true,   // Include omni.ja defaults
//!     include_globals: true,    // Include greprefs.js
//!     include_user: true,       // Include user prefs.js
//!     continue_on_error: true,  // Don't fail if some sources are missing
//! };
//!
//! let merged = merge_all_preferences(&profile_path, None, &config)?;
//!
//! println!("Loaded {} preferences from {} sources",
//!     merged.entries.len(),
//!     merged.loaded_sources.len()
//! );
//!
//! // Display warnings for any missing sources
//! for warning in &merged.warnings {
//!     eprintln!("Warning: {}", warning);
//! }
//! # Ok::<(), ffcv::Error>(())
//! ```
//!
//! ### Filtering User-Modified Preferences
//!
//! ```rust,no_run
//! use ffcv::{merge_all_preferences, find_profile_path, MergeConfig, PrefSource};
//!
//! let profile_path = find_profile_path("default-release", None)?;
//! let config = MergeConfig::default();
//! let merged = merge_all_preferences(&profile_path, None, &config)?;
//!
//! // Get only user-modified preferences (exclude built-ins)
//! let user_modified: Vec<_> = merged.entries
//!     .iter()
//!     .filter(|p| p.source == Some(PrefSource::User))
//!     .collect();
//!
//! println!("You've modified {} preferences", user_modified.len());
//! # Ok::<(), ffcv::Error>(())
//! ```
//!
//! ## Preference Types
//!
//! Firefox supports four preference types, indicated by the function name used:
//!
//! - **user_pref()** - [`PrefType::User`]: User-set preferences (most common)
//! - **pref()** - [`PrefType::Default`]: Application defaults
//! - **lock_pref()** - [`PrefType::Locked`]: Locked preferences (cannot be changed by users)
//! - **sticky_pref()** - [`PrefType::Sticky`]: Sticky preferences (persist across updates)
//!
//! ## Error Handling
//!
//! All functions return [`Result<T, Error>`]. The [`Error`] enum provides
//! detailed error context for programmatic handling:
//!
//! ```rust
//! use ffcv::{parse_prefs_js, Error};
//!
//! let invalid_content = "user_pref(invalid syntax";
//! match parse_prefs_js(invalid_content) {
//!     Ok(config) => println!("Parsed successfully"),
//!     Err(Error::Parser { line, column, message }) => {
//!         eprintln!("Parse error at {}:{}: {}", line, column, message);
//!     }
//!     Err(e) => eprintln!("Other error: {}", e),
//! }
//! ```
//!
//! ## Platform Support
//!
//! This library automatically detects Firefox profiles on:
//! - **Linux**: `~/.mozilla/firefox/`
//! - **macOS**: `~/Library/Application Support/Firefox/`
//! - **Windows**: `%APPDATA%\Mozilla\Firefox\Profiles\`
//!
//! ## See Also
//!
//! - [Firefox preferences documentation](https://support.mozilla.org/en-US/kb/about-config-editor-firefox)
//! - [prefs.js format reference](https://searchfox.org/mozilla-central/source/modules/libpref/parser/src/lib.rs)

// Re-export all public types at crate root
pub use types::{
    FirefoxInstallation, MergedPreferences, PrefEntry, PrefSource, PrefType, PrefValue,
    PrefValueExt,
};

// Re-export error types
pub use error::{Error, Result};

// Re-export profile types
pub use profile::ProfileInfo;

// Re-export all public functions at crate root
pub use parser::{parse_prefs_js, parse_prefs_js_file};
pub use profile::{find_profile_path, get_prefs_path, list_profiles};
pub use query::query_preferences;

// Re-export Firefox locator
pub use firefox_locator::{
    find_all_firefox_installations, find_firefox_installation, get_firefox_version,
};

// Re-export omni_extractor
pub use omni_extractor::{ExtractConfig, OmniExtractor, DEFAULT_MAX_OMNI_SIZE};

// Re-export pref_merger
pub use pref_merger::{get_effective_pref, merge_all_preferences, MergeConfig};

// All modules are private - use re-exports above for public API
mod error;
mod explanations;
mod firefox_locator;
mod lexer;
mod omni_extractor;
mod parser;
mod pref_merger;
mod profile;
mod query;
mod types;
