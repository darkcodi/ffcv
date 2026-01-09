// Backward Compatibility Tests
//
// This test file ensures that existing API continues to work as expected
// after adding Firefox defaults functionality.

use std::path::PathBuf;

#[test]
fn test_parse_prefs_js_basic_api() {
    // Test that basic parsing API works exactly as before
    use ffcv::{parse_prefs_js, PrefType, PrefValue};

    let content = r#"
        user_pref("browser.startup.homepage", "https://example.com");
        user_pref("javascript.enabled", true);
        user_pref("network.proxy.type", 1);
    "#;

    let prefs = parse_prefs_js(content).expect("Failed to parse preferences");

    assert_eq!(prefs.len(), 3);

    let homepage = prefs
        .iter()
        .find(|e| e.key == "browser.startup.homepage")
        .unwrap();
    assert_eq!(
        homepage.value,
        PrefValue::String("https://example.com".to_string())
    );
    assert_eq!(homepage.pref_type, PrefType::User);

    let js_enabled = prefs
        .iter()
        .find(|e| e.key == "javascript.enabled")
        .unwrap();
    assert_eq!(js_enabled.value, PrefValue::Bool(true));

    let proxy_type = prefs
        .iter()
        .find(|e| e.key == "network.proxy.type")
        .unwrap();
    assert_eq!(proxy_type.value, PrefValue::Integer(1));
}

#[test]
fn test_parse_prefs_js_file_basic_api() {
    // Test that file-based parsing API works
    use ffcv::parse_prefs_js_file;

    let fixture_path = PathBuf::from("tests/fixtures/firefox-esr115/prefs.js");

    let prefs = parse_prefs_js_file(&fixture_path).expect("Failed to parse prefs.js");

    assert!(!prefs.is_empty());

    // Verify that all parsed preferences have source information
    for pref in &prefs {
        assert!(pref.source.is_some(), "All prefs should have source info");
        assert!(
            pref.source_file.is_some(),
            "All prefs should have source file info"
        );
    }
}

#[test]
fn test_query_preferences_basic_api() {
    // Test that query API works exactly as before
    use ffcv::{parse_prefs_js, query_preferences};

    let content = r#"
        user_pref("network.proxy.http", "proxy.example.com");
        user_pref("network.proxy.http_port", 8080);
        user_pref("network.proxy.type", 1);
        user_pref("browser.startup.homepage", "https://example.com");
    "#;

    let prefs = parse_prefs_js(content).expect("Failed to parse");

    let network_prefs = query_preferences(&prefs, &["network.*"]).expect("Failed to query");

    assert_eq!(network_prefs.len(), 3);
    assert!(network_prefs.iter().all(|p| p.key.starts_with("network.")));
}

#[test]
fn test_query_multiple_patterns() {
    // Test that multiple pattern query works
    use ffcv::{parse_prefs_js, query_preferences};

    let content = r#"
        user_pref("network.proxy.type", 1);
        user_pref("browser.startup.page", 3);
        user_pref("extensions.enabled", true);
    "#;

    let prefs = parse_prefs_js(content).expect("Failed to parse");

    let results = query_preferences(&prefs, &["network.*", "browser.*"]).expect("Failed to query");

    assert_eq!(results.len(), 2);
}

#[test]
fn test_pref_value_convenience_methods() {
    // Test that PrefValueExt trait methods work
    use ffcv::{parse_prefs_js, PrefValue, PrefValueExt};

    let content = r#"
        user_pref("bool.pref", true);
        user_pref("int.pref", 42);
        user_pref("float.pref", 3.14);
        user_pref("string.pref", "hello");
    "#;

    let prefs = parse_prefs_js(content).expect("Failed to parse");

    for pref in &prefs {
        match &pref.value {
            PrefValue::Bool(b) => {
                let extracted = pref.value.as_bool().unwrap();
                assert_eq!(extracted, *b);
            }
            PrefValue::Integer(i) => {
                let extracted = pref.value.as_i64().unwrap();
                assert_eq!(extracted, *i);
            }
            PrefValue::Float(f) => {
                let extracted = pref.value.as_f64().unwrap();
                assert!((extracted - *f).abs() < 0.0001);
            }
            PrefValue::String(s) => {
                let extracted = pref.value.as_str().unwrap();
                assert_eq!(extracted, *s);
            }
            PrefValue::Null => {}
        }
    }
}

#[test]
fn test_pref_entry_find_by_key() {
    // Test that find_by_key helper works
    use ffcv::{parse_prefs_js, PrefEntry};

    let content = r#"
        user_pref("test.pref", "value");
        user_pref("another.pref", 123);
    "#;

    let prefs = parse_prefs_js(content).expect("Failed to parse");

    let found = PrefEntry::find_by_key(&prefs, "test.pref");
    assert!(found.is_some());
    assert_eq!(found.unwrap().key, "test.pref");

    let not_found = PrefEntry::find_by_key(&prefs, "nonexistent");
    assert!(not_found.is_none());
}

#[test]
fn test_json_serialization() {
    // Test that PrefEntry still serializes to JSON correctly
    use ffcv::parse_prefs_js;

    let content = r#"user_pref("test.pref", "value");"#;

    let prefs = parse_prefs_js(content).expect("Failed to parse");
    let pref = &prefs[0];

    // Serialize to JSON - should not fail
    let json_str = serde_json::to_string(pref).expect("Failed to serialize to string");

    // Verify JSON string contains expected content
    assert!(json_str.contains("test.pref"));
    assert!(json_str.contains("value"));

    // Verify it's valid JSON by parsing it back
    let _parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse serialized JSON");
}

#[test]
fn test_pref_source_enum() {
    // Test that PrefSource enum is properly exported and works
    use ffcv::PrefSource;

    // Test all enum variants
    let _builtin = PrefSource::BuiltIn;
    let _global = PrefSource::GlobalDefault;
    let user = PrefSource::User;
    let _policy = PrefSource::SystemPolicy;

    // Test equality
    assert_eq!(user, PrefSource::User);
    assert_ne!(user, PrefSource::BuiltIn);

    // Test that it can be serialized
    let json = serde_json::to_string(&user).expect("Failed to serialize");
    assert_eq!(json, "\"user\"");
}

#[test]
fn test_new_types_compile() {
    // Test that new types compile and can be instantiated
    use ffcv::{FirefoxInstallation, MergedPreferences, PrefSource};
    use std::path::PathBuf;

    // FirefoxInstallation
    let installation = FirefoxInstallation {
        version: "115.0".to_string(),
        path: PathBuf::from("/usr/lib/firefox"),
        has_greprefs: true,
        has_omni_ja: true,
    };
    assert_eq!(installation.version, "115.0");

    // MergedPreferences
    let merged = MergedPreferences {
        entries: vec![],
        install_path: Some(PathBuf::from("/usr/lib/firefox")),
        profile_path: PathBuf::from("/home/user/.mozilla/firefox/profile"),
        loaded_sources: vec![PrefSource::User, PrefSource::BuiltIn],
        warnings: vec![],
    };
    assert_eq!(merged.loaded_sources.len(), 2);
}

#[test]
fn test_error_type_still_works() {
    // Test that Error enum still works with existing variants
    use ffcv::{parse_prefs_js, Error};

    // Test with lexer error (unclosed string) which still fails
    let invalid = r#"user_pref("unclosed string)"#;

    match parse_prefs_js(invalid) {
        Err(Error::Parser {
            line,
            column,
            message,
        }) => {
            assert!(line > 0);
            assert!(column > 0);
            assert!(!message.is_empty());
        }
        _ => panic!("Expected parser error for unclosed string"),
    }
}

#[test]
fn test_display_impl_still_works() {
    // Test that Display impl for PrefEntry works
    use ffcv::parse_prefs_js;

    let content = r#"user_pref("test.pref", true);"#;
    let prefs = parse_prefs_js(content).expect("Failed to parse");
    let pref = &prefs[0];

    let display_str = format!("{}", pref);
    assert!(display_str.contains("test.pref"));
    assert!(display_str.contains("true"));
}

#[test]
fn test_debug_impl_still_works() {
    // Test that Debug impl for PrefEntry works
    use ffcv::parse_prefs_js;

    let content = r#"user_pref("test.pref", 42);"#;
    let prefs = parse_prefs_js(content).expect("Failed to parse");
    let pref = &prefs[0];

    let debug_str = format!("{:?}", pref);
    assert!(debug_str.contains("test.pref"));
}
