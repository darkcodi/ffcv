use regex::Regex;
use std::collections::HashMap;

/// Parse the prefs.js file and extract all preferences
pub fn parse_prefs_js(content: &str) -> Result<HashMap<String, serde_json::Value>, anyhow::Error> {
    let mut preferences = HashMap::new();

    // Regex pattern to match user_pref("key", value);
    let regex = Regex::new(r#"user_pref\(\s*"([^"]+)"\s*,\s*(.+)\);"#)
        .map_err(|e| anyhow::anyhow!("Failed to compile regex: {}", e))?;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if let Some(caps) = regex.captures(line) {
            let key = caps[1].to_string();
            let value_str = caps[2].trim();

            match parse_value(value_str) {
                Ok(value) => {
                    preferences.insert(key, value);
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to parse value for key '{}': {}",
                        key,
                        e
                    ));
                }
            }
        }
    }

    Ok(preferences)
}

/// Parse a JavaScript value to serde_json::Value
fn parse_value(value_str: &str) -> Result<serde_json::Value, anyhow::Error> {
    let value_str = value_str.trim();

    // String value (quoted)
    if value_str.starts_with('"') && value_str.ends_with('"') {
        let content = &value_str[1..value_str.len() - 1];
        // Handle escaped characters
        let unescaped = content.replace("\\\"", "\"");
        return Ok(serde_json::Value::String(unescaped));
    }

    // Boolean values
    if value_str == "true" {
        return Ok(serde_json::Value::Bool(true));
    }
    if value_str == "false" {
        return Ok(serde_json::Value::Bool(false));
    }

    // Null value
    if value_str == "null" {
        return Ok(serde_json::Value::Null);
    }

    // Numeric values
    if let Ok(int_val) = value_str.parse::<i64>() {
        return Ok(serde_json::Value::Number(serde_json::Number::from(int_val)));
    }

    if let Ok(float_val) = value_str.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(float_val) {
            return Ok(serde_json::Value::Number(num));
        }
    }

    // If we can't parse it, treat as string
    Ok(serde_json::Value::String(value_str.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_prefs_js_string() {
        let input = r#"user_pref("browser.startup.homepage", "https://example.com");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["browser.startup.homepage"],
            serde_json::Value::String("https://example.com".to_string())
        );
    }

    #[test]
    fn test_parse_prefs_js_boolean() {
        let input = r#"user_pref("javascript.enabled", true);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result["javascript.enabled"], serde_json::Value::Bool(true));
    }

    #[test]
    fn test_parse_prefs_js_integer() {
        let input = r#"user_pref("network.cookie.cookieBehavior", 0);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["network.cookie.cookieBehavior"],
            serde_json::Value::Number(serde_json::Number::from(0))
        );
    }

    #[test]
    fn test_parse_prefs_js_multiple() {
        let input = r#"
            user_pref("browser.startup.homepage", "https://example.com");
            user_pref("javascript.enabled", true);
            user_pref("network.cookie.cookieBehavior", 0);
        "#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_parse_prefs_js_with_comments() {
        let input = r#"
            // This is a comment
            user_pref("browser.startup.homepage", "https://example.com");
            // Another comment
            user_pref("javascript.enabled", true);
        "#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_value_string_with_escaped_quotes() {
        let input = r#"user_pref("test", "value with \"quotes\"");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["test"],
            serde_json::Value::String("value with \"quotes\"".to_string())
        );
    }

    #[test]
    fn test_parse_value_float() {
        let result = parse_value("3.14").unwrap();
        assert!(result.is_number());
    }

    #[test]
    fn test_parse_value_null() {
        let result = parse_value("null").unwrap();
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn test_parse_value_unknown() {
        let result = parse_value("undefined").unwrap();
        assert_eq!(result, serde_json::Value::String("undefined".to_string()));
    }
}
