use crate::cli;
use crate::profile::{find_profile_path, get_prefs_path, list_profiles as list_profiles_impl};
use crate::query;
use crate::types::Config;

/// List all available Firefox profiles
pub fn list_profiles(
    profiles_dir_opt: Option<&std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = list_profiles_impl(profiles_dir_opt).map_err(|e| {
        anyhow::anyhow!(
            "Failed to list profiles: {}. Make sure Firefox is installed.",
            e
        )
    })?;

    let json = serde_json::to_string_pretty(&profiles)?;
    println!("{}", json);
    Ok(())
}

/// View configuration for a specific profile
pub fn view_config(
    profile_name: &str,
    profiles_dir_opt: Option<&std::path::Path>,
    query_patterns: &[&str],
    get: Option<String>,
    output_type: cli::OutputType,
    unexplained_only: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let profile_path = find_profile_path(profile_name, profiles_dir_opt).map_err(|e| {
        anyhow::anyhow!(
            "Failed to find profile '{}': {}. Make sure Firefox is installed and the profile exists.\n\
             Use 'ffcv profile' to see available profiles.",
            profile_name,
            e
        )
    })?;

    let prefs_path = get_prefs_path(&profile_path);
    let content = std::fs::read_to_string(&prefs_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to read prefs.js at {}: {e}. Make sure the file exists and is readable.",
            prefs_path.display()
        )
    })?;

    // Parse preferences with or without type information based on output type
    let (preferences, preferences_with_types): (
        Config,
        Option<std::collections::HashMap<String, crate::types::PrefEntry>>,
    ) = match output_type {
        cli::OutputType::JsonObject => {
            // Use standard parser for json-object (no type info needed)
            let prefs = crate::parser::parse_prefs_js(&content).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse prefs.js: {e}. The file may be corrupted or in an unexpected format."
                )
            })?;
            (prefs, None)
        }
        cli::OutputType::JsonArray => {
            // Use parser with type info for json-array
            let prefs_with_types = crate::parser::parse_prefs_js_with_types(&content).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse prefs.js: {e}. The file may be corrupted or in an unexpected format."
                )
            })?;
            // Also create Config for query filtering
            let prefs: Config = prefs_with_types
                .iter()
                .map(|(k, v)| (k.clone(), v.value.clone()))
                .collect();
            (prefs, Some(prefs_with_types))
        }
    };

    // Handle --get mode: single preference retrieval with raw output
    if let Some(get_key) = get {
        if let Some(value) = preferences.get(&get_key) {
            // Check unexplained-only flag
            if unexplained_only {
                let has_explanation = crate::types::get_preference_explanation(&get_key).is_some();
                if has_explanation {
                    return Err(anyhow::anyhow!(
                        "Preference '{}' has an explanation, but --unexplained-only was specified",
                        get_key
                    )
                    .into());
                }
            }
            output_raw_value(value)?;
            return Ok(());
        }
        // If preference not found, return error
        return Err(anyhow::anyhow!("Preference '{}' not found", get_key).into());
    }

    // Apply queries if provided
    let mut output_config = if !query_patterns.is_empty() {
        query::query_preferences(&preferences, query_patterns)
            .map_err(|e| anyhow::anyhow!("Failed to apply query: {}", e))?
    } else {
        preferences
    };

    // Apply unexplained-only filter if flag is set
    if unexplained_only {
        output_config.retain(|key, _| {
            // Keep only preferences that don't have explanations
            crate::types::get_preference_explanation(key).is_none()
        });
    }

    let json = match output_type {
        cli::OutputType::JsonObject => serde_json::to_string_pretty(&output_config)?,
        cli::OutputType::JsonArray => {
            // Use the type information we parsed earlier
            let prefs_with_types = preferences_with_types
                .as_ref()
                .expect("Type info should be available for json-array");
            let array_output: Vec<crate::types::ConfigEntry> = output_config
                .iter()
                .map(|(key, value)| {
                    // Look up the type information
                    let pref_type = prefs_with_types
                        .get(key)
                        .map(|entry| entry.pref_type.clone());
                    let explanation = crate::types::get_preference_explanation(key);
                    crate::types::ConfigEntry {
                        key: key.clone(),
                        value: value.clone(),
                        pref_type,
                        explanation,
                    }
                })
                .collect();
            serde_json::to_string_pretty(&array_output)?
        }
    };

    println!("{}", json);
    Ok(())
}

/// Output a single preference value in raw format (no JSON wrapping)
fn output_raw_value(value: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    match value {
        serde_json::Value::String(s) => println!("{}", s),
        serde_json::Value::Bool(b) => println!("{}", b),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                println!("{}", n.as_i64().unwrap());
            } else {
                println!("{}", n.as_f64().unwrap());
            }
        }
        serde_json::Value::Null => println!("null"),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            // Complex types still output as JSON
            println!("{}", value);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::types::PrefType;
    use serde_json::json;

    /// Helper function to test the output formatting logic
    fn format_value(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    format!("{}", n.as_i64().unwrap())
                } else {
                    format!("{}", n.as_f64().unwrap())
                }
            }
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Bool(b) => format!("{}", b),
            serde_json::Value::Null => "null".to_string(),
            _ => value.to_string(),
        }
    }

    #[test]
    fn test_config_entry_serialization_with_pref_type() {
        // Test that ConfigEntry serializes correctly with pref_type
        let entry = crate::types::ConfigEntry {
            key: "test.key".to_string(),
            value: json!("test value"),
            pref_type: Some(PrefType::User),
            explanation: None,
        };

        let json_str = serde_json::to_string(&entry).unwrap();
        assert!(json_str.contains("\"pref_type\":\"user\""));
        assert!(json_str.contains("\"key\":\"test.key\""));
        assert!(json_str.contains("\"value\":\"test value\""));
        // explanation should not be present when None
        assert!(!json_str.contains("explanation"));
    }

    #[test]
    fn test_config_entry_serialization_without_pref_type() {
        // Test that ConfigEntry serializes correctly without pref_type (None)
        let entry = crate::types::ConfigEntry {
            key: "test.key".to_string(),
            value: json!("test value"),
            pref_type: None,
            explanation: None,
        };

        let json_str = serde_json::to_string(&entry).unwrap();
        // pref_type should not be in the output when None (due to skip_serializing_if)
        assert!(!json_str.contains("pref_type"));
        assert!(json_str.contains("\"key\":\"test.key\""));
        assert!(json_str.contains("\"value\":\"test value\""));
        // explanation should also not be present when None
        assert!(!json_str.contains("explanation"));
    }

    #[test]
    fn test_pref_type_serialization() {
        // Test all pref type variants serialize correctly
        let tests = vec![
            (PrefType::User, "user"),
            (PrefType::Default, "default"),
            (PrefType::Locked, "locked"),
            (PrefType::Sticky, "sticky"),
        ];

        for (pref_type, expected_str) in tests {
            let json_str = serde_json::to_string(&pref_type).unwrap();
            assert_eq!(json_str, format!("\"{}\"", expected_str));
        }
    }

    #[test]
    fn test_json_array_output_with_types() {
        // Test that json-array output includes pref_type for all entries
        let input = r#"
            user_pref("user.pref", "value1");
            pref("default.pref", "value2");
            lock_pref("locked.pref", "value3");
            sticky_pref("sticky.pref", "value4");
        "#;

        let prefs_with_types = crate::parser::parse_prefs_js_with_types(input).unwrap();
        let array_output: Vec<crate::types::ConfigEntry> = prefs_with_types
            .iter()
            .map(|(key, entry)| crate::types::ConfigEntry {
                key: key.clone(),
                value: entry.value.clone(),
                pref_type: Some(entry.pref_type.clone()),
                explanation: crate::types::get_preference_explanation(key),
            })
            .collect();

        let json_str = serde_json::to_string_pretty(&array_output).unwrap();

        // Verify pref_type field is present for all entries
        assert!(json_str.contains("pref_type"));
        assert!(json_str.contains("\"user\""));
        assert!(json_str.contains("\"default\""));
        assert!(json_str.contains("\"locked\""));
        assert!(json_str.contains("\"sticky\""));

        // Verify keys are present
        assert!(json_str.contains("user.pref"));
        assert!(json_str.contains("default.pref"));
        assert!(json_str.contains("locked.pref"));
        assert!(json_str.contains("sticky.pref"));

        // Verify structure (should have key, value, pref_type for each entry)
        // explanation should NOT be present since these prefs don't have explanations
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.len(), 4);
        for entry in parsed {
            assert!(entry.is_object());
            let obj = entry.as_object().unwrap();
            assert!(obj.contains_key("key"));
            assert!(obj.contains_key("value"));
            assert!(obj.contains_key("pref_type"));
            // explanation field should not be present for these unexplained prefs
            assert!(!obj.contains_key("explanation"));
        }
    }

    #[test]
    fn test_output_raw_value_integer() {
        let value = json!(3);
        let output = format_value(&value);
        assert_eq!(output, "3");
        assert!(!output.contains('.'));
    }

    #[test]
    fn test_output_raw_value_negative_integer() {
        let value = json!(-42);
        let output = format_value(&value);
        assert_eq!(output, "-42");
        assert!(!output.contains('.'));
    }

    #[test]
    fn test_output_raw_value_zero() {
        let value = json!(0);
        let output = format_value(&value);
        assert_eq!(output, "0");
        assert!(!output.contains('.'));
    }

    #[test]
    fn test_output_raw_value_float() {
        let value = json!(3.14);
        let output = format_value(&value);
        assert_eq!(output, "3.14");
        assert!(output.contains('.'));
    }

    #[test]
    fn test_output_raw_value_float_whole_number() {
        let value = serde_json::Value::Number(serde_json::Number::from_f64(3.0).unwrap());
        let output = format_value(&value);
        assert_eq!(output, "3");
        // This is the key test: a whole number float should display as integer
        assert!(!output.contains('.'));
    }

    #[test]
    fn test_output_raw_value_string() {
        let value = json!("test value");
        let output = format_value(&value);
        assert_eq!(output, "test value");
    }

    #[test]
    fn test_output_raw_value_bool() {
        let value = json!(true);
        let output = format_value(&value);
        assert_eq!(output, "true");

        let value = json!(false);
        let output = format_value(&value);
        assert_eq!(output, "false");
    }

    #[test]
    fn test_output_raw_value_null() {
        let value = json!(null);
        let output = format_value(&value);
        assert_eq!(output, "null");
    }

    #[test]
    fn test_config_entry_serialization_with_explanation() {
        // Test that ConfigEntry includes explanation field in JSON output
        let entry = crate::types::ConfigEntry {
            key: "javascript.enabled".to_string(),
            value: json!(true),
            pref_type: Some(PrefType::Default),
            explanation: Some(
                "Master switch to enable or disable JavaScript execution.".to_string(),
            ),
        };

        let json_str = serde_json::to_string(&entry).unwrap();
        assert!(json_str.contains("\"explanation\":"));
        assert!(json_str.contains("Master switch to enable or disable JavaScript execution"));
    }

    #[test]
    fn test_config_entry_serialization_without_explanation() {
        // Test that ConfigEntry without explanation does not include the field
        let entry = crate::types::ConfigEntry {
            key: "unknown.pref".to_string(),
            value: json!("test"),
            pref_type: None,
            explanation: None,
        };

        let json_str = serde_json::to_string(&entry).unwrap();
        // explanation field should not be in output when None
        assert!(!json_str.contains("explanation"));
    }

    #[test]
    fn test_json_array_output_includes_explanations() {
        // Test full pipeline with explanations
        let input = r#"
            user_pref("javascript.enabled", true);
            user_pref("browser.startup.homepage", "https://example.com");
        "#;

        let prefs_with_types = crate::parser::parse_prefs_js_with_types(input).unwrap();
        let array_output: Vec<crate::types::ConfigEntry> = prefs_with_types
            .iter()
            .map(|(key, entry)| crate::types::ConfigEntry {
                key: key.clone(),
                value: entry.value.clone(),
                pref_type: Some(entry.pref_type.clone()),
                explanation: crate::types::get_preference_explanation(key),
            })
            .collect();

        let json_str = serde_json::to_string_pretty(&array_output).unwrap();

        // Verify javascript.enabled has its explanation
        assert!(json_str.contains("Master switch to enable or disable JavaScript"));

        // Verify entries are handled correctly
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.len(), 2);

        // First entry (javascript.enabled) should have explanation
        let js_entry = parsed[0].as_object().unwrap();
        assert!(js_entry.contains_key("explanation"));

        // Second entry (browser.startup.homepage) should NOT have explanation
        let homepage_entry = parsed[1].as_object().unwrap();
        assert!(!homepage_entry.contains_key("explanation"));
    }
}
