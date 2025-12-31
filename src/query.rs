use crate::types::Config;
use glob::Pattern;

/// Query configuration preferences by glob patterns (OR logic)
/// Returns preferences matching any of the provided patterns
pub fn query_preferences(preferences: &Config, patterns: &[&str]) -> Result<Config, anyhow::Error> {
    // Compile all patterns first to fail fast on invalid patterns
    let compiled_patterns: Vec<Pattern> = patterns
        .iter()
        .map(|p| {
            Pattern::new(p).map_err(|e| anyhow::anyhow!("Invalid query pattern '{}': {}", p, e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Query preferences: keep if ANY pattern matches
    let queried: Config = preferences
        .iter()
        .filter(|(key, _)| compiled_patterns.iter().any(|pattern| pattern.matches(key)))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();

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
