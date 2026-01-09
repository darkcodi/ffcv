//! Query and filtering operations for Firefox preferences
//!
//! This module provides functionality for filtering preferences using glob patterns.
//!
//! # Example
//!
//! ```rust
//! use ffcv::{parse_prefs_js, query_preferences, PrefEntry, PrefType};
//!
//! let content = r#"
//!     user_pref("network.proxy.http", "proxy.example.com");
//!     user_pref("network.proxy.http_port", 8080);
//!     user_pref("browser.startup.homepage", "https://example.com");
//! "#;
//!
//! let prefs = parse_prefs_js(content)?;
//! let network_prefs = query_preferences(&prefs, &["network.*"])?;
//! assert_eq!(network_prefs.len(), 2);
//! # Ok::<(), ffcv::Error>(())
//! ```

use crate::error::{Error, Result};
use crate::types::PrefEntry;
use glob::Pattern;

/// Query configuration preferences by glob patterns (OR logic)
///
/// Returns preferences matching any of the provided patterns. Patterns use
/// standard glob syntax (e.g., "network.*", "browser.*.enabled").
///
/// # Arguments
///
/// * `preferences` - Slice of preference entries to query
/// * `patterns` - Slice of glob patterns to match against (OR logic)
///
/// # Example
///
/// ```rust
/// use ffcv::{query_preferences, PrefEntry, PrefType, PrefValue, PrefSource};
///
/// let prefs = vec![
///     PrefEntry {
///         key: "network.proxy.http".to_string(),
///         value: PrefValue::String("proxy.example.com".to_string()),
///         pref_type: PrefType::User,
///         explanation: None,
///         source: Some(PrefSource::User),
///         source_file: Some("prefs.js".to_string()),
///         locked: None,
///     },
///     PrefEntry {
///         key: "browser.startup.homepage".to_string(),
///         value: PrefValue::String("https://example.com".to_string()),
///         pref_type: PrefType::User,
///         explanation: None,
///         source: Some(PrefSource::User),
///         source_file: Some("prefs.js".to_string()),
///         locked: None,
///     },
/// ];
///
/// let network_prefs = query_preferences(&prefs, &["network.*"])?;
/// assert_eq!(network_prefs.len(), 1);
/// # Ok::<(), ffcv::Error>(())
/// ```
pub fn query_preferences(preferences: &[PrefEntry], patterns: &[&str]) -> Result<Vec<PrefEntry>> {
    // Compile all patterns first to fail fast on invalid patterns
    let compiled_patterns: Vec<Pattern> = patterns
        .iter()
        .map(|p| {
            Pattern::new(p).map_err(|e| {
                Error::InvalidGlobPattern(format!("Invalid query pattern '{}': {}", p, e))
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Filter preferences: keep if ANY pattern matches
    let queried: Vec<PrefEntry> = preferences
        .iter()
        .filter(|entry| {
            compiled_patterns
                .iter()
                .any(|pattern| pattern.matches(&entry.key))
        })
        .cloned()
        .collect();

    Ok(queried)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PrefSource;
    use crate::types::{PrefType, PrefValue};

    fn create_test_config() -> Vec<PrefEntry> {
        vec![
            PrefEntry {
                key: "network.proxy.type".to_string(),
                value: PrefValue::Integer(1),
                pref_type: PrefType::User,
                explanation: None,
                source: Some(PrefSource::User),
                source_file: Some("prefs.js".to_string()),
                locked: None,
            },
            PrefEntry {
                key: "network.cookie.cookieBehavior".to_string(),
                value: PrefValue::Integer(0),
                pref_type: PrefType::User,
                explanation: None,
                source: Some(PrefSource::User),
                source_file: Some("prefs.js".to_string()),
                locked: None,
            },
            PrefEntry {
                key: "browser.startup.homepage".to_string(),
                value: PrefValue::String("https://example.com".to_string()),
                pref_type: PrefType::User,
                explanation: None,
                source: Some(PrefSource::User),
                source_file: Some("prefs.js".to_string()),
                locked: None,
            },
            PrefEntry {
                key: "browser.search.region".to_string(),
                value: PrefValue::String("US".to_string()),
                pref_type: PrefType::User,
                explanation: None,
                source: Some(PrefSource::User),
                source_file: Some("prefs.js".to_string()),
                locked: None,
            },
            PrefEntry {
                key: "javascript.enabled".to_string(),
                value: PrefValue::Bool(true),
                pref_type: PrefType::Default,
                explanation: None,
                source: Some(PrefSource::User),
                source_file: Some("prefs.js".to_string()),
                locked: None,
            },
        ]
    }

    #[test]
    fn test_query_single_pattern() {
        let config = create_test_config();
        let queried = query_preferences(&config, &["network.*"]).unwrap();
        assert_eq!(queried.len(), 2);
        assert!(queried.iter().any(|e| e.key == "network.proxy.type"));
        assert!(queried
            .iter()
            .any(|e| e.key == "network.cookie.cookieBehavior"));
    }

    #[test]
    fn test_query_multiple_patterns_or_logic() {
        let config = create_test_config();
        let queried = query_preferences(&config, &["network.*", "javascript.enabled"]).unwrap();
        assert_eq!(queried.len(), 3);
        assert!(queried.iter().any(|e| e.key == "network.proxy.type"));
        assert!(queried
            .iter()
            .any(|e| e.key == "network.cookie.cookieBehavior"));
        assert!(queried.iter().any(|e| e.key == "javascript.enabled"));
    }

    #[test]
    fn test_query_no_matches() {
        let config = create_test_config();
        let queried = query_preferences(&config, &["nonexistent.*"]).unwrap();
        assert_eq!(queried.len(), 0);
        assert!(queried.is_empty());
    }

    #[test]
    fn test_query_invalid_pattern() {
        let config = create_test_config();
        let result = query_preferences(&config, &["[invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_query_exact_match() {
        let config = create_test_config();
        let queried = query_preferences(&config, &["javascript.enabled"]).unwrap();
        assert_eq!(queried.len(), 1);
        assert!(queried.iter().any(|e| e.key == "javascript.enabled"));
    }
}
