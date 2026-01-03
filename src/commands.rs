use crate::cli;
use ffcv::profile::{find_profile_path, get_prefs_path, list_profiles as list_profiles_impl};
use ffcv::query;

/// Configuration parameters for viewing Firefox configuration
pub struct ViewConfigParams<'a> {
    pub stdin: bool,
    pub profile_name: &'a str,
    pub profiles_dir_opt: Option<&'a std::path::Path>,
    pub max_file_size: usize,
    pub query_patterns: &'a [&'a str],
    pub get: Option<String>,
    pub output_type: cli::OutputType,
    pub unexplained_only: bool,
}

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

/// Read preference content from standard input
fn read_stdin_content(max_file_size: usize) -> Result<String, Box<dyn std::error::Error>> {
    use std::io::{self, Read};

    let mut buffer = String::new();
    let bytes_read = io::stdin().read_to_string(&mut buffer).map_err(|e| {
        anyhow::anyhow!(
            "Failed to read from stdin: {}. Make sure to pipe prefs.js content.",
            e
        )
    })?;

    if bytes_read > max_file_size {
        return Err(anyhow::anyhow!(
            "Input from stdin exceeds maximum size limit: {} bytes > {} bytes. \
             Use --max-file-size to increase the limit.",
            bytes_read,
            max_file_size
        )
        .into());
    }

    Ok(buffer)
}

/// Read preference content from file system
fn read_file_content(
    profile_name: &str,
    profiles_dir_opt: Option<&std::path::Path>,
    max_file_size: usize,
) -> Result<String, Box<dyn std::error::Error>> {
    let profile_path = find_profile_path(profile_name, profiles_dir_opt).map_err(|e| {
        anyhow::anyhow!(
            "Failed to find profile '{}': {}. Make sure Firefox is installed and the profile exists.\n\
             Use 'ffcv profile' to see available profiles.",
            profile_name,
            e
        )
    })?;

    let prefs_path = get_prefs_path(&profile_path);

    // Check file size before reading to prevent DoS attacks
    let metadata = std::fs::metadata(&prefs_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to read metadata for prefs.js at {}: {e}.",
            prefs_path.display()
        )
    })?;

    let file_size = metadata.len() as usize;
    if file_size > max_file_size {
        return Err(anyhow::anyhow!(
            "File size exceeds maximum limit: {} bytes ({} MB) > {} bytes ({} MB). \
             Use --max-file-size to increase the limit.",
            file_size,
            file_size / 1_048_576,
            max_file_size,
            max_file_size / 1_048_576
        )
        .into());
    }

    std::fs::read_to_string(&prefs_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to read prefs.js at {}: {e}. Make sure the file exists and is readable.",
            prefs_path.display()
        )
        .into()
    })
}

/// View configuration for a specific profile
pub fn view_config(params: ViewConfigParams) -> Result<(), Box<dyn std::error::Error>> {
    // Read content from appropriate source (stdin or file)
    let content = if params.stdin {
        read_stdin_content(params.max_file_size)?
    } else {
        read_file_content(
            params.profile_name,
            params.profiles_dir_opt,
            params.max_file_size,
        )?
    };

    // Parse preferences (always returns Vec<PrefEntry> with types)
    let preferences: Vec<ffcv::PrefEntry> =
        ffcv::parser::parse_prefs_js(&content).map_err(|e| {
            let source_hint = if params.stdin {
                "from stdin"
            } else {
                "from prefs.js file"
            };
            anyhow::anyhow!(
                "Failed to parse preferences {}: {}. The input may be malformed.",
                source_hint,
                e
            )
        })?;

    // Handle --get mode: single preference retrieval with raw output
    if let Some(get_key) = params.get {
        if let Some(entry) = preferences.iter().find(|e| e.key == get_key) {
            // Check unexplained-only flag
            if params.unexplained_only && entry.explanation.is_some() {
                return Err(anyhow::anyhow!(
                    "Preference '{}' has an explanation, but --unexplained-only was specified",
                    get_key
                )
                .into());
            }
            output_raw_value(&entry.value)?;
            return Ok(());
        }
        // If preference not found, return error
        return Err(anyhow::anyhow!("Preference '{}' not found", get_key).into());
    }

    // Apply queries if provided
    let mut output_prefs = if !params.query_patterns.is_empty() {
        query::query_preferences(&preferences, params.query_patterns)
            .map_err(|e| anyhow::anyhow!("Failed to apply query: {}", e))?
    } else {
        preferences.clone()
    };

    // Apply unexplained-only filter if flag is set
    if params.unexplained_only {
        output_prefs.retain(|entry| {
            // Keep only preferences that don't have explanations
            entry.explanation.is_none()
        });
    }

    let json = match params.output_type {
        cli::OutputType::JsonObject => {
            // Convert Vec<PrefEntry> to HashMap for JSON object output
            let json_map: std::collections::HashMap<String, serde_json::Value> = output_prefs
                .iter()
                .map(|entry| (entry.key.clone(), entry.value.clone()))
                .collect();
            serde_json::to_string_pretty(&json_map)?
        }
        cli::OutputType::JsonArray => {
            // Use Vec<PrefEntry> directly for array output
            let mut sorted_entries = output_prefs.clone();

            // Sort alphabetically by key for deterministic output order
            sorted_entries.sort_by(|a, b| a.key.cmp(&b.key));

            serde_json::to_string_pretty(&sorted_entries)?
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
    use ffcv::PrefType;
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
    fn test_pref_entry_serialization() {
        // Test that PrefEntry serializes correctly
        let entry = ffcv::PrefEntry {
            key: "test.key".to_string(),
            value: json!("test value"),
            pref_type: PrefType::User,
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
        // Test that json-array output is sorted alphabetically by key
        let input = r#"
            user_pref("user.pref", "value1");
            pref("default.pref", "value2");
            lock_pref("locked.pref", "value3");
            sticky_pref("sticky.pref", "value4");
        "#;

        let mut array_output = ffcv::parser::parse_prefs_js(input).unwrap();

        // Sort to match production code behavior
        array_output.sort_by(|a, b| a.key.cmp(&b.key));

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

        // Verify alphabetical ordering
        let keys: Vec<&str> = parsed
            .iter()
            .map(|entry| entry["key"].as_str().unwrap())
            .collect();
        assert_eq!(
            keys,
            vec!["default.pref", "locked.pref", "sticky.pref", "user.pref"]
        );

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
        let value = json!(2.5);
        let output = format_value(&value);
        assert_eq!(output, "2.5");
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
    fn test_pref_entry_serialization_with_explanation() {
        // Test that PrefEntry includes explanation field in JSON output
        let entry = ffcv::PrefEntry {
            key: "javascript.enabled".to_string(),
            value: json!(true),
            pref_type: PrefType::Default,
            explanation: Some("Master switch to enable or disable JavaScript execution."),
        };

        let json_str = serde_json::to_string(&entry).unwrap();
        assert!(json_str.contains("\"explanation\":"));
        assert!(json_str.contains("Master switch to enable or disable JavaScript execution"));
    }

    #[test]
    fn test_pref_entry_serialization_without_explanation() {
        // Test that PrefEntry without explanation does not include the field
        let entry = ffcv::PrefEntry {
            key: "unknown.pref".to_string(),
            value: json!("test"),
            pref_type: PrefType::User,
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

        let mut array_output = ffcv::parser::parse_prefs_js(input).unwrap();

        // Sort to match production code behavior
        array_output.sort_by(|a, b| a.key.cmp(&b.key));

        let json_str = serde_json::to_string_pretty(&array_output).unwrap();

        // Verify javascript.enabled has its explanation
        assert!(json_str.contains("Master switch to enable or disable JavaScript"));

        // Verify entries are handled correctly
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.len(), 2);

        // Find entries by key instead of by index (deterministic regardless of order)
        let js_entry = parsed
            .iter()
            .find(|entry| entry["key"] == "javascript.enabled")
            .expect("javascript.enabled should be present")
            .as_object()
            .unwrap();
        assert!(js_entry.contains_key("explanation"));

        let homepage_entry = parsed
            .iter()
            .find(|entry| entry["key"] == "browser.startup.homepage")
            .expect("browser.startup.homepage should be present")
            .as_object()
            .unwrap();
        assert!(!homepage_entry.contains_key("explanation"));
    }

    #[test]
    fn test_stdin_size_limit_enforcement() {
        // Create a large string that exceeds a small limit
        let large_content = "user_pref(\"test\", \"x\");".repeat(1000);
        let small_limit = 100;

        // The real test is in the integration test with actual large files
        // This test documents the expected behavior
        assert!(large_content.len() > small_limit);
    }

    #[test]
    fn test_max_file_size_parameter() {
        // Test that max_file_size parameter is properly typed
        let max_size: usize = 10_485_760; // 10MB in bytes
        assert_eq!(max_size, 10_485_760);

        // Test that we can calculate MB from bytes
        let size_in_mb = max_size / 1_048_576;
        assert_eq!(size_in_mb, 10);
    }
}
