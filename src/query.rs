//! Query and filtering operations for Firefox preferences
//!
//! This module provides functionality for filtering preferences using glob patterns.
//!
//! # Example
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
//! let config = parse_prefs_js(content)?;
//! let network_prefs = query_preferences(&config, &["network.*"])?;
//! assert_eq!(network_prefs.len(), 2);
//! # Ok::<(), ffcv::Error>(())
//! ```

use crate::error::{Error, Result};
use crate::types::Config;
use glob::Pattern;

/// Query configuration preferences by glob patterns (OR logic)
///
/// Returns preferences matching any of the provided patterns. Patterns use
/// standard glob syntax (e.g., "network.*", "browser.*.enabled").
///
/// # Arguments
///
/// * `preferences` - The configuration map to query
/// * `patterns` - Slice of glob patterns to match against (OR logic)
///
/// # Example
///
/// ```rust
/// use ffcv::query_preferences;
/// use serde_json::json;
///
/// let mut config = std::collections::HashMap::new();
/// config.insert("network.proxy.http".to_string(), json!("proxy.example.com"));
/// config.insert("browser.startup.homepage".to_string(), json!("https://example.com"));
///
/// let network_prefs = query_preferences(&config, &["network.*"])?;
/// assert_eq!(network_prefs.len(), 1);
/// # Ok::<(), ffcv::Error>(())
/// ```
pub fn query_preferences(preferences: &Config, patterns: &[&str]) -> Result<Config> {
    // Compile all patterns first to fail fast on invalid patterns
    let compiled_patterns: Vec<Pattern> = patterns
        .iter()
        .map(|p| {
            Pattern::new(p).map_err(|e| {
                Error::InvalidGlobPattern(format!("Invalid query pattern '{}': {}", p, e))
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Query preferences: keep if ANY pattern matches
    // First count matching entries to pre-allocate HashMap capacity
    let matching_count = preferences
        .iter()
        .filter(|(key, _)| compiled_patterns.iter().any(|pattern| pattern.matches(key)))
        .count();

    // Pre-allocate HashMap with exact capacity to avoid reallocations
    let mut queried = Config::with_capacity(matching_count);
    for (key, value) in preferences.iter() {
        if compiled_patterns.iter().any(|pattern| pattern.matches(key)) {
            queried.insert(key.clone(), value.clone());
        }
    }

    Ok(queried)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn create_test_config() -> Config {
        let mut config = Config::new();
        config.insert("network.proxy.type".to_string(), Value::Number(1.into()));
        config.insert(
            "network.cookie.cookieBehavior".to_string(),
            Value::Number(0.into()),
        );
        config.insert(
            "browser.startup.homepage".to_string(),
            Value::String("https://example.com".to_string()),
        );
        config.insert(
            "browser.search.region".to_string(),
            Value::String("US".to_string()),
        );
        config.insert("javascript.enabled".to_string(), Value::Bool(true));
        config
    }

    #[test]
    fn test_query_single_pattern() {
        let config = create_test_config();
        let queried = query_preferences(&config, &["network.*"]).unwrap();
        assert_eq!(queried.len(), 2);
        assert!(queried.contains_key("network.proxy.type"));
        assert!(queried.contains_key("network.cookie.cookieBehavior"));
    }

    #[test]
    fn test_query_multiple_patterns_or_logic() {
        let config = create_test_config();
        let queried = query_preferences(&config, &["network.*", "javascript.enabled"]).unwrap();
        assert_eq!(queried.len(), 3);
        assert!(queried.contains_key("network.proxy.type"));
        assert!(queried.contains_key("network.cookie.cookieBehavior"));
        assert!(queried.contains_key("javascript.enabled"));
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
        assert!(queried.contains_key("javascript.enabled"));
    }
}
