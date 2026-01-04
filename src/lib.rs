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
//! - Detect and manage Firefox profiles across platforms (Linux, macOS, Windows)
//! - Query preferences using glob patterns (e.g., `"network.*"`, `"browser.*.enabled"`)
//! - Get human-readable explanations for documented preferences
//! - Support for all four preference types: user, default, locked, and sticky
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

// Re-export core types at crate root for convenient access
pub use profile::ProfileInfo;
pub use types::{PrefEntry, PrefType, PrefValue, PrefValueExt};

// Re-export error types
pub use error::{Error, Result};

// Public modules
pub mod parser;
pub mod profile;
pub mod query;

// Re-export key functions at crate level for ergonomic API
pub use parser::{parse_prefs_js, parse_prefs_js_file};
pub use profile::{find_profile_path, get_prefs_path, list_profiles};
pub use query::query_preferences;

// Internal modules (private)
mod error;
mod explanations;
mod lexer;
mod types;
