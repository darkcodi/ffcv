# Test Fixtures for Firefox Defaults

This directory contains test fixtures for validating Firefox default preferences extraction.

## Directory Structure

```
fixtures/
├── firefox-esr115/
│   ├── omni-ja-esr115.ja          # Simulated omni.ja file
│   ├── greprefs.js                 # Global default preferences
│   └── prefs.js                    # User preferences
└── firefox-release128/
    ├── omni-ja-release128.ja       # Simulated omni.ja file
    ├── greprefs.js                 # Global default preferences
    └── prefs.js                    # User preferences
```

## Fixture Details

### Firefox ESR 115
- **Version**: 115.0esr
- **omni.ja contents**:
  - `defaults/pref/browser.js` - Browser-specific built-in defaults
  - `defaults/pref/firefox.js` - General Firefox built-in defaults
- **greprefs.js**: Global preferences with higher precedence than built-ins
- **prefs.js**: User-modified preferences (highest precedence)

### Firefox Release 128
- **Version**: 128.0
- **omni.ja contents**:
  - `defaults/pref/browser.js` - Browser-specific built-in defaults
  - `defaults/pref/firefox.js` - General Firefox built-in defaults
- **greprefs.js**: Global preferences
- **prefs.js**: User-modified preferences

## Test Scenarios Covered

### Preference Precedence Testing
The fixtures include overlapping preferences across sources to test merge precedence:

1. **app.update.auto**
   - Built-in: `true` (omni.ja)
   - Global: `true` (greprefs.js)
   - User: `false` (prefs.js)
   - Expected final value: `false` (user wins)

2. **browser.startup.homepage**
   - Built-in: `about:blank`
   - Global: `about:blank`
   - User: `https://example.com`
   - Expected final value: `https://example.com`

3. **network.proxy.type**
   - Built-in: `0`
   - Global: `0`
   - User: `1` (ESR115) or `0` (Release128)
   - Expected final value: `1` (ESR115), `0` (Release128)

4. **toolkit.telemetry.enabled**
   - Built-in: `true`
   - Global: `false`
   - User: `false`
   - Expected final value: `false` (global and user agree)

### Type Mismatch Testing
Some preferences have the same key but different types to test type conflict handling.

### Source Tracking Verification
Each fixture includes known values to verify that source tracking works correctly.

## Using These Fixtures

### In Unit Tests
```rust
use std::path::PathBuf;

let fixture_path = PathBuf::from("tests/fixtures/firefox-esr115");
let omni_path = fixture_path.join("omni-ja-esr115.ja");
```

### In Integration Tests
```rust
use ffcv::merge_all_preferences;
use ffcv::MergeConfig;

let config = MergeConfig::default();
let merged = merge_all_preferences(
    &profile_path,
    Some(&install_path),
    &config
)?;
```

## Notes

- These are **simulated** omni.ja files for testing purposes
- Real omni.ja files are much larger (50-100MB)
- Real omni.ja files contain many more .js files
- The structure matches real Firefox installations for testing extraction logic

## Creating Real Fixtures

To create fixtures from real Firefox installations:

```bash
# On Linux
cp /usr/lib/firefox/omni.ja tests/fixtures/firefox-real/omni.ja
cp /usr/lib/firefox/greprefs.js tests/fixtures/firefox-real/greprefs.js

# On macOS
cp /Applications/Firefox.app/Contents/Resources/omni.ja tests/fixtures/
```

**Warning**: Real omni.ja files may contain proprietary Firefox code. Only use for local testing, do not commit to public repositories.
