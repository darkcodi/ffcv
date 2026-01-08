//! Core type definitions for Firefox preferences
//!
//! This module defines the data structures used throughout the ffcv library
//! for representing Firefox preferences and their metadata.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// Firefox preference value types
///
/// Firefox preferences only support a limited set of primitive types.
/// Complex data structures (arrays, objects) are stored as JSON strings,
/// not as native types.
///
/// # Example
///
/// ```rust
/// use ffcv::{PrefValue, PrefValueExt};
///
/// let bool_val = PrefValue::Bool(true);
/// let int_val = PrefValue::Integer(42);
/// let string_val = PrefValue::String("example".to_string());
///
/// assert_eq!(bool_val.as_bool(), Some(true));
/// assert_eq!(int_val.as_i64(), Some(42));
/// assert_eq!(string_val.as_str(), Some("example"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrefValue {
    /// Boolean value (true/false)
    Bool(bool),
    /// Integer value (64-bit for safety, though Firefox uses 32-bit)
    Integer(i64),
    /// Floating-point value
    Float(f64),
    /// String value (including JSON-encoded complex data)
    String(String),
    /// Null value
    Null,
}

impl PrefValue {
    /// Convert f64 to Integer or Float variant
    ///
    /// Whole numbers without fractional parts become Integer(i64),
    /// numbers with fractional parts become Float(f64).
    pub fn from_f64(num: f64) -> Self {
        if num.fract() == 0.0 && num >= i64::MIN as f64 && num <= i64::MAX as f64 {
            PrefValue::Integer(num as i64)
        } else {
            PrefValue::Float(num)
        }
    }
}

impl fmt::Display for PrefValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrefValue::Bool(b) => write!(f, "{}", b),
            PrefValue::Integer(i) => write!(f, "{}", i),
            PrefValue::Float(float) => write!(f, "{}", float),
            PrefValue::String(s) => write!(f, "\"{}\"", s),
            PrefValue::Null => write!(f, "null"),
        }
    }
}

/// Extension trait providing convenience methods for PrefValue
///
/// This trait provides ergonomic accessor methods for working with
/// preference values without pattern matching.
///
/// **Note:** To use this trait's methods, you must import both `PrefValue`
/// and `PrefValueExt`:
///
/// ```rust
/// use ffcv::{PrefValue, PrefValueExt};
/// ```
///
/// # Example
///
/// ```rust
/// use ffcv::{PrefValue, PrefValueExt};
///
/// let value = PrefValue::String("test".to_string());
///
/// if let Some(s) = value.as_str() {
///     println!("String value: {}", s);
/// }
///
/// assert_eq!(value.type_name(), "String");
/// assert!(!value.is_null());
/// ```
pub trait PrefValueExt {
    /// Returns the value as a bool if it is a Bool variant
    fn as_bool(&self) -> Option<bool>;

    /// Returns the value as an i64 if it is an Integer variant
    fn as_i64(&self) -> Option<i64>;

    /// Returns the value as an f64 if it is an Integer or Float variant
    fn as_f64(&self) -> Option<f64>;

    /// Returns the value as a string slice if it is a String variant
    fn as_str(&self) -> Option<&str>;

    /// Returns true if the value is Null
    fn is_null(&self) -> bool;

    /// Returns true if the value is an Integer or Float
    fn is_number(&self) -> bool;

    /// Returns the type name as a static string
    fn type_name(&self) -> &'static str;

    /// Converts PrefValue to serde_json::Value for compatibility
    fn to_json_value(&self) -> serde_json::Value;
}

impl PrefValueExt for PrefValue {
    fn as_bool(&self) -> Option<bool> {
        match self {
            PrefValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            PrefValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            PrefValue::Integer(i) => Some(*i as f64),
            PrefValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            PrefValue::String(s) => Some(s),
            _ => None,
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, PrefValue::Null)
    }

    fn is_number(&self) -> bool {
        matches!(self, PrefValue::Integer(_) | PrefValue::Float(_))
    }

    fn type_name(&self) -> &'static str {
        match self {
            PrefValue::Bool(_) => "Bool",
            PrefValue::Integer(_) => "Integer",
            PrefValue::Float(_) => "Float",
            PrefValue::String(_) => "String",
            PrefValue::Null => "Null",
        }
    }

    fn to_json_value(&self) -> serde_json::Value {
        match self {
            PrefValue::Bool(b) => serde_json::Value::Bool(*b),
            PrefValue::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            PrefValue::Float(f) => serde_json::Value::Number(
                serde_json::Number::from_f64(*f).unwrap_or_else(|| serde_json::Number::from(0)),
            ),
            PrefValue::String(s) => serde_json::Value::String(s.clone()),
            PrefValue::Null => serde_json::Value::Null,
        }
    }
}

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

/// Firefox preference source
///
/// Indicates where a preference value originates from in the Firefox
/// configuration hierarchy. This helps users understand the provenance
/// of their configuration settings.
///
/// Firefox loads preferences in a specific order, with later sources
/// overriding earlier ones:
///
/// 1. Built-in defaults (omni.ja) - Lowest precedence
/// 2. Global defaults (greprefs.js) - Medium precedence
/// 3. User preferences (prefs.js) - Highest precedence
/// 4. System policies (policies.json) - Overrides all (future)
///
/// # Example
///
/// ```rust
/// use ffcv::PrefSource;
///
/// let source = PrefSource::BuiltIn;
/// assert_eq!(source, PrefSource::BuiltIn);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrefSource {
    /// Built-in default from omni.ja (e.g., defaults/pref/*.js)
    #[serde(rename = "builtin")]
    BuiltIn,
    /// Global default from greprefs.js in Firefox installation directory
    #[serde(rename = "global")]
    GlobalDefault,
    /// User preference from prefs.js in profile directory
    #[serde(rename = "user")]
    User,
    /// System-wide policy from policies.json (future support)
    #[serde(rename = "policy")]
    SystemPolicy,
}

/// Internal type for parser that always includes pref type
///
/// This structure is returned by `parse_prefs_js()` and contains
/// both the preference value and its type.
///
/// # Example
///
/// ```rust
/// use ffcv::{parse_prefs_js, PrefType, PrefValue};
///
/// let content = r#"user_pref("test", true);"#;
/// let prefs = parse_prefs_js(content)?;
/// let entry = prefs.iter().find(|e| e.key == "test").unwrap();
/// assert_eq!(entry.key, "test");
/// assert_eq!(entry.pref_type, PrefType::User);
/// assert_eq!(entry.value, PrefValue::Bool(true));
/// # Ok::<(), ffcv::Error>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefEntry {
    /// The preference name/key
    pub key: String,
    /// The preference value
    pub value: PrefValue,
    /// The type of preference (user, default, locked, sticky)
    pub pref_type: PrefType,
    /// Optional human-readable explanation for the preference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<&'static str>,
    /// The source of this preference value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<PrefSource>,
    /// The origin file for this preference (e.g., "prefs.js", "omni.ja:defaults/pref/browser.js")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
}

impl PrefEntry {
    /// Find the first preference entry matching the given key
    ///
    /// This is a convenience method for finding preferences by key without
    /// manually iterating through the slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ffcv::{parse_prefs_js, PrefEntry};
    ///
    /// let content = r#"
    ///     user_pref("browser.startup.homepage", "https://example.com");
    ///     user_pref("javascript.enabled", true);
    /// "#;
    /// let prefs = parse_prefs_js(content)?;
    ///
    /// if let Some(entry) = PrefEntry::find_by_key(&prefs, "browser.startup.homepage") {
    ///     println!("Found homepage: {:?}", entry.value);
    /// }
    /// # Ok::<(), ffcv::Error>(())
    /// ```
    pub fn find_by_key<'a>(prefs: &'a [PrefEntry], key: &str) -> Option<&'a PrefEntry> {
        prefs.iter().find(|e| e.key == key)
    }
}

impl fmt::Display for PrefEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let source_info = if let Some(ref source) = self.source {
            match source {
                PrefSource::BuiltIn => " [builtin]",
                PrefSource::GlobalDefault => " [global]",
                PrefSource::User => " [user]",
                PrefSource::SystemPolicy => " [policy]",
            }
        } else {
            ""
        };

        write!(
            f,
            "{}({:?}){} = {}",
            self.key, self.pref_type, source_info, self.value
        )
    }
}

/// Firefox installation information
///
/// Represents a Firefox installation directory with metadata about
/// available configuration files.
///
/// # Example
///
/// ```rust
/// use ffcv::FirefoxInstallation;
/// use std::path::PathBuf;
///
/// let install = FirefoxInstallation {
///     version: "128.0".to_string(),
///     path: PathBuf::from("/usr/lib/firefox"),
///     has_greprefs: true,
///     has_omni_ja: true,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct FirefoxInstallation {
    /// Firefox version string (e.g., "128.0", "115.5.0esr")
    pub version: String,
    /// Path to Firefox installation directory
    pub path: PathBuf,
    /// Whether greprefs.js exists in this installation
    pub has_greprefs: bool,
    /// Whether omni.ja exists in this installation
    pub has_omni_ja: bool,
}

/// Merged preferences from multiple sources
///
/// Represents the result of merging preferences from built-in defaults,
/// global defaults, and user preferences, along with metadata about
/// which sources were successfully loaded.
///
/// # Example
///
/// ```rust
/// use ffcv::{MergedPreferences, PrefEntry, PrefSource};
/// use std::path::PathBuf;
///
/// let merged = MergedPreferences {
///     entries: vec![],
///     install_path: Some(PathBuf::from("/usr/lib/firefox")),
///     profile_path: PathBuf::from("/home/user/.mozilla/firefox/default"),
///     loaded_sources: vec![PrefSource::User],
///     warnings: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct MergedPreferences {
    /// All merged preference entries (with highest precedence value for each key)
    pub entries: Vec<PrefEntry>,
    /// Path to Firefox installation (if found)
    pub install_path: Option<PathBuf>,
    /// Path to Firefox profile directory
    pub profile_path: PathBuf,
    /// Which preference sources were successfully loaded
    pub loaded_sources: Vec<PrefSource>,
    /// Any warnings or issues encountered during merging
    pub warnings: Vec<String>,
}
