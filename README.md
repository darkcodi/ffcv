# ffcv

[![crates.io](https://img.shields.io/crates/v/ffcv)](https://crates.io/crates/ffcv)
[![docs.rs](https://img.shields.io/docsrs/ffcv)](https://docs.rs/ffcv)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE-MIT.txt)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

**Firefox Configuration Viewer** - Parse and query Firefox preference files with ease.

ffcv is both a command-line tool and a Rust library for working with Firefox's `prefs.js` configuration files. It provides a structured way to parse, query, and understand Firefox settings across all platforms (Linux, macOS, Windows).

## Features

- **Complete Parsing** - Handles all Firefox preference types:
  - User preferences (`user_pref`)
  - Default preferences (`pref`)
  - Locked preferences (`lock_pref`)
  - Sticky preferences (`sticky_pref`)
- **Powerful Querying** - Filter preferences using glob patterns like `"network.*"` or `"browser.*.enabled"`
- **Cross-Platform** - Automatic Firefox profile discovery on Linux, macOS, and Windows
- **Rich Data Types** - Supports boolean, integer, float, string, and null values
- **Human-Readable Explanations** - Optional explanations for what preferences do
- **Flexible Output** - JSON output with customizable formatting
- **Well-Tested** - Comprehensive test coverage with robust error handling

## Installation

### Command-Line Tool

Install via cargo:

```bash
cargo install ffcv
```

This installs the `ffcv` binary for viewing Firefox configurations from your terminal.

### Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
ffcv = "0.1"
```

## Command-Line Usage

### List Firefox Profiles

```bash
# List all Firefox profiles on your system
ffcv profile list

# Specify a custom Firefox profiles directory
ffcv profile list --profiles-dir /custom/path
```

### View Configuration

```bash
# View all preferences for the default profile
ffcv config view default

# Query specific preferences by glob pattern
ffcv config view default --query "network.*"
ffcv config view default --query "browser.*" "extensions.*"

# Get a single preference
ffcv config get default "network.proxy.type"

# Read from stdin
cat prefs.js | ffcv config view -

# Output as JSON array
ffcv config view default --output-format array

# Include human-readable explanations
ffcv config view default --include-explanations

# Read from a specific file
ffcv config view default --file /path/to/prefs.js
```

## Library Usage

### Basic Parsing

```rust
use ffcv::parser::parse_prefs;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string("prefs.js")?;
    let entries = parse_prefs(&content)?;

    for entry in entries {
        println!("{:?} = {:?}", entry.key, entry.value);
    }

    Ok(())
}
```

### Query Preferences

```rust
use ffcv::parser::parse_prefs;
use ffcv::query::QueryMatcher;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string("prefs.js")?;
    let entries = parse_prefs(&content)?;

    // Create a matcher for network-related preferences
    let matcher = QueryMatcher::new(vec!["network.*"])?;

    // Filter and display matching entries
    for entry in entries.into_iter().filter(|e| matcher.matches(&e.key)) {
        println!("{} = {:?}", entry.key, entry.value);
    }

    Ok(())
}
```

### Profile Discovery

```rust
use ffcv::profile::ProfileFinder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let finder = ProfileFinder::new()?;
    let profiles = finder.find_profiles()?;

    for profile in profiles {
        println!("Profile: {}", profile.name);
        println!("  Path: {}", profile.path.display());
        println!("  Default: {}", profile.is_default);
    }

    Ok(())
}
```

### Working with Preference Values

```rust
use ffcv::types::{PrefEntry, PrefValue};

fn process_entry(entry: PrefEntry) {
    match entry.value {
        PrefValue::Bool(true) => println!("{} is enabled", entry.key),
        PrefValue::Bool(false) => println!("{} is disabled", entry.key),
        PrefValue::Int(n) => println!("{} = {} (integer)", entry.key, n),
        PrefValue::String(ref s) => println!("{} = \"{}\"", entry.key, s),
        _ => println!("{} = {:?}", entry.key, entry.value),
    }
}
```

## Preference Types

Firefox uses several types of preferences:

- **User Preferences** (`user_pref`) - Set by the user
- **Default Preferences** (`pref`) - Application defaults
- **Locked Preferences** (`lock_pref`) - Administratively locked, cannot be changed
- **Sticky Preferences** (`sticky_pref`) - User preferences that persist across updates

Each preference has a value type:
- Boolean (`true`/`false`)
- Integer (64-bit)
- Float (64-bit)
- String (including JSON-encoded data)
- Null

## Environment Variables

ffcv respects the following environment variables for Firefox profile discovery:

- `FIREFOX_BIN` - Path to Firefox binary
- `MOZ_PROFILES_DIR` - Custom Firefox profiles directory
- `PROGRAMFILES` - Windows Program Files directory (for Firefox detection)
- `HOME` - User home directory (Unix-like systems)
- `APPDATA` - Application Data directory (Windows)

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/ffcv).

To build and view the documentation locally:

```bash
cargo doc --open
```

## Platform Support

ffcv is tested and supported on:
- **Linux** (x86_64, ARM64)
- **macOS** (x86_64, ARM64/Apple Silicon)
- **Windows** (x86_64)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT.txt](LICENSE-MIT.txt) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE-2.0.txt](LICENSE-APACHE-2.0.txt) or http://www.apache.org/licenses/LICENSE-2.0)

You may choose either license for your use.

## Acknowledgments

Built for the Rust community to make Firefox configuration management easier and more programmatic.

Inspired by the need for better tools to understand and manage Firefox's extensive preference system.
