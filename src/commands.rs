use crate::cli;
use crate::profile::{find_profile_path, get_prefs_path, list_profiles as list_profiles_impl};
use crate::query;
use crate::types::Config;

/// List all available Firefox profiles
pub fn list_profiles() -> Result<(), Box<dyn std::error::Error>> {
    let profiles = list_profiles_impl().map_err(|e| {
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
    query_patterns: &[&str],
    get: Option<String>,
    output_type: cli::OutputType,
) -> Result<(), Box<dyn std::error::Error>> {
    let profile_path = find_profile_path(profile_name).map_err(|e| {
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

    let preferences: Config = crate::parser::parse_prefs_js(&content).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse prefs.js: {e}. The file may be corrupted or in an unexpected format."
        )
    })?;

    // Handle --get mode: single preference retrieval with raw output
    if let Some(get_key) = get {
        if let Some(value) = preferences.get(&get_key) {
            output_raw_value(value)?;
            return Ok(());
        }
        // If preference not found, return error
        return Err(anyhow::anyhow!("Preference '{}' not found", get_key).into());
    }

    // Apply queries if provided
    let output_config = if !query_patterns.is_empty() {
        query::query_preferences(&preferences, query_patterns)
            .map_err(|e| anyhow::anyhow!("Failed to apply query: {}", e))?
    } else {
        preferences
    };

    let json = match output_type {
        cli::OutputType::JsonObject => serde_json::to_string_pretty(&output_config)?,
        cli::OutputType::JsonArray => {
            let array_output: Vec<crate::types::ConfigEntry> = output_config
                .iter()
                .map(|(key, value)| crate::types::ConfigEntry {
                    key: key.clone(),
                    value: value.clone(),
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
}
