// Integration tests for Firefox defaults feature
use ffcv::PrefValueExt;
use std::path::{Path, PathBuf};

// Helper function to get fixtures path
fn fixtures_path() -> PathBuf {
    Path::new("tests/fixtures").to_path_buf()
}

#[test]
fn test_omni_extraction_from_fixture() {
    use ffcv::{parse_prefs_js_file, ExtractConfig, OmniExtractor};

    let omni_path = fixtures_path()
        .join("firefox-esr115")
        .join("omni-ja-esr115.ja");

    assert!(omni_path.exists(), "omni.ja fixture should exist");

    let config = ExtractConfig {
        max_omni_size: 10_000_000,
        cache_dir: None,
        target_files: vec!["defaults/pref/*.js".to_string()],
        force_refresh: false,
    };

    let extractor =
        OmniExtractor::with_config(omni_path, config).expect("Failed to create extractor");

    let extracted_files = extractor
        .extract_prefs()
        .expect("Failed to extract preferences");

    // Should extract preferences from both browser.js and firefox.js
    assert!(
        !extracted_files.is_empty(),
        "Should extract at least some files"
    );

    // Parse the extracted files to verify they contain valid preferences
    let mut all_prefs = Vec::new();
    for file_path in extracted_files {
        let prefs = parse_prefs_js_file(&file_path)
            .unwrap_or_else(|_| panic!("Failed to parse {:?}", file_path));
        all_prefs.extend(prefs);
    }

    // Verify we got some preferences
    assert!(
        !all_prefs.is_empty(),
        "Parsed preferences should not be empty"
    );

    // Verify specific built-in preferences exist
    let browser_startpage_exists = all_prefs.iter().any(|p| p.key.contains("startup"));
    assert!(
        browser_startpage_exists,
        "Should extract startup-related preferences from defaults/pref/browser.js"
    );
}

#[test]
fn test_preference_precedence() {
    use ffcv::{parse_prefs_js_file, query_preferences, PrefSource};

    let prefs_path = fixtures_path().join("firefox-esr115").join("prefs.js");

    let entries = parse_prefs_js_file(&prefs_path).expect("Failed to parse prefs.js");

    // Verify specific preference values from our fixture
    let network_proxy_type = query_preferences(&entries, &["network.proxy.type"])
        .expect("Failed to query network.proxy.type");

    assert_eq!(network_proxy_type.len(), 1);
    assert_eq!(network_proxy_type[0].key, "network.proxy.type");

    // In our fixture, network.proxy.type should be 1 (user_pref)
    if let Some(value) = network_proxy_type[0].value.as_i64() {
        assert_eq!(value, 1);
    } else {
        panic!("network.proxy.type should be an integer");
    }

    // Verify source is set to User
    assert_eq!(network_proxy_type[0].source, Some(PrefSource::User));
}

#[test]
fn test_source_tracking_in_parsed_prefs() {
    use ffcv::{parse_prefs_js_file, PrefSource};

    let prefs_path = fixtures_path().join("firefox-release128").join("prefs.js");

    let entries = parse_prefs_js_file(&prefs_path).expect("Failed to parse prefs.js");

    // All parsed preferences should have PrefSource::User
    for entry in &entries {
        assert_eq!(entry.source, Some(PrefSource::User));
        assert_eq!(entry.source_file, Some("prefs.js".to_string()));
    }
}

#[test]
fn test_fixture_files_are_valid() {
    // Test that all fixture files can be parsed
    use ffcv::parse_prefs_js_file;

    let fixtures = vec![
        fixtures_path().join("firefox-esr115").join("prefs.js"),
        fixtures_path().join("firefox-esr115").join("greprefs.js"),
        fixtures_path().join("firefox-release128").join("prefs.js"),
        fixtures_path()
            .join("firefox-release128")
            .join("greprefs.js"),
    ];

    for fixture_path in fixtures {
        assert!(
            fixture_path.exists(),
            "Fixture file should exist: {:?}",
            fixture_path
        );

        let result = parse_prefs_js_file(&fixture_path);
        assert!(
            result.is_ok(),
            "Should successfully parse {:?}",
            fixture_path
        );

        let entries = result.unwrap();
        assert!(
            !entries.is_empty(),
            "{:?} should contain preferences",
            fixture_path
        );
    }
}

#[test]
fn test_omni_extraction_with_invalid_file() {
    use ffcv::Error;
    use ffcv::OmniExtractor;

    let invalid_path = fixtures_path().join("nonexistent.ja");

    let result = OmniExtractor::new(invalid_path);

    assert!(result.is_err(), "Should fail with nonexistent omni.ja");

    // The error could be OmniJaError or PrefFileNotFound
    match result {
        Err(Error::OmniJaError(_)) | Err(Error::PrefFileNotFound { .. }) => {
            // Expected error types
        }
        _ => {
            panic!("Expected OmniJaError or PrefFileNotFound for nonexistent file");
        }
    }
}

#[test]
fn test_query_with_glob_patterns() {
    use ffcv::{parse_prefs_js_file, query_preferences};

    let prefs_path = fixtures_path().join("firefox-esr115").join("prefs.js");

    let entries = parse_prefs_js_file(&prefs_path).expect("Failed to parse prefs.js");

    // Test querying with glob patterns
    let network_prefs =
        query_preferences(&entries, &["network.*"]).expect("Failed to query network.*");

    assert!(
        !network_prefs.is_empty(),
        "Should find network.* preferences"
    );

    for pref in &network_prefs {
        assert!(pref.key.starts_with("network."));
    }

    // Test multiple patterns
    let browser_and_network = query_preferences(&entries, &["browser.*", "network.*"])
        .expect("Failed to query multiple patterns");

    assert!(!browser_and_network.is_empty());

    for pref in &browser_and_network {
        assert!(
            pref.key.starts_with("browser.") || pref.key.starts_with("network."),
            "All results should match one of the patterns"
        );
    }
}

#[test]
fn test_backward_compatibility_existing_api() {
    use ffcv::{parse_prefs_js_file, query_preferences, PrefValueExt};

    let prefs_path = fixtures_path().join("firefox-esr115").join("prefs.js");

    // Test that existing API functions still work
    let entries = parse_prefs_js_file(&prefs_path).expect("Failed to parse prefs.js");

    // Test query_preferences
    let results = query_preferences(&entries, &["browser.*"]).expect("Failed to query");

    assert!(!results.is_empty());

    // Test PrefValue convenience methods
    for entry in &entries {
        // These methods should all work without errors
        let _ = entry.value.as_bool();
        let _ = entry.value.as_i64();
        let _ = entry.value.as_f64();
        let _ = entry.value.as_str();
    }
}

#[test]
fn test_merged_preferences_structure() {
    use ffcv::{MergedPreferences, PrefSource};
    use std::path::PathBuf;

    // Create a MergedPreferences structure to verify it works
    let merged = MergedPreferences {
        entries: vec![],
        install_path: Some(PathBuf::from("/usr/lib/firefox")),
        profile_path: PathBuf::from("/home/user/.mozilla/firefox/profile"),
        loaded_sources: vec![
            PrefSource::BuiltIn,
            PrefSource::GlobalDefault,
            PrefSource::User,
        ],
        warnings: vec![],
    };

    // Verify structure is valid
    assert_eq!(merged.loaded_sources.len(), 3);
    assert!(merged.install_path.is_some());
    assert!(merged.warnings.is_empty());
}
