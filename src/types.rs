//! Core type definitions for Firefox preferences
//!
//! This module defines the data structures used throughout the ffcv library
//! for representing Firefox preferences and their metadata.

use serde::{Deserialize, Serialize};

// Re-export the explanation function for convenience
pub use crate::explanations::get_preference_explanation;

/// Firefox preference types
///
/// Firefox supports four different preference types, indicated by which
/// function was used to set the preference in prefs.js:
///
/// - [`PrefType::User`] - Set by user via `user_pref()`
/// - [`PrefType::Default`] - Application default via `pref()`
/// - [`PrefType::Locked`] - Locked by administrator via `lock_pref()`
/// - [`PrefType::Sticky`] - Sticky preference via `sticky_pref()`
///
/// # Example
///
/// ```rust
/// use ffcv::PrefType;
///
/// let pref_type = PrefType::User;
/// assert_eq!(pref_type, PrefType::User);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrefType {
    /// User-set preference (set via `user_pref()`)
    #[serde(rename = "user")]
    User,
    /// Application default preference (set via `pref()`)
    #[serde(rename = "default")]
    Default,
    /// Locked preference that cannot be changed by users (set via `lock_pref()`)
    #[serde(rename = "locked")]
    Locked,
    /// Sticky preference that persists across updates (set via `sticky_pref()`)
    #[serde(rename = "sticky")]
    Sticky,
}

/// Internal type for parser that always includes pref type
///
/// This structure is returned by `parse_prefs_js()` and contains
/// both the preference value and its type.
///
/// # Example
///
/// ```rust
/// use ffcv::{parse_prefs_js, PrefType};
///
/// let content = r#"user_pref("test", true);"#;
/// let prefs = parse_prefs_js(content)?;
/// let entry = prefs.iter().find(|e| e.key == "test").unwrap();
/// assert_eq!(entry.key, "test");
/// assert_eq!(entry.pref_type, PrefType::User);
/// # Ok::<(), ffcv::Error>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefEntry {
    /// The preference name/key
    pub key: String,
    /// The preference value
    pub value: serde_json::Value,
    /// The type of preference (user, default, locked, sticky)
    pub pref_type: PrefType,
}

/// Representation for array output format
///
/// This structure is used when outputting preferences in JSON array format,
/// providing additional metadata such as type and explanation.
///
/// # Fields
///
/// * `key` - The preference name
/// * `value` - The preference value
/// * `pref_type` - Optional type information (user, default, locked, sticky)
/// * `explanation` - Optional human-readable explanation
///
/// # Example
///
/// ```rust
/// use ffcv::ConfigEntry;
/// use serde_json::json;
///
/// let entry = ConfigEntry {
///     key: "javascript.enabled".to_string(),
///     value: json!(true),
///     pref_type: Some(ffcv::PrefType::Default),
///     explanation: Some("Master switch for JavaScript".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct ConfigEntry {
    /// The preference name/key
    pub key: String,
    /// The preference value
    pub value: serde_json::Value,
    /// Optional type information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pref_type: Option<PrefType>,
    /// Optional human-readable explanation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}
