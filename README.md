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
- **Type-Safe API** - Convenience trait for easy value type checking and extraction
- **Simple Interface** - All public types and functions available at crate root
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
ffcv = "1.0.1"
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
use ffcv::parse_prefs_js;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string("prefs.js")?;
    let entries = parse_prefs_js(&content)?;

    for entry in entries {
        println!("{:?} = {:?}", entry.key, entry.value);
    }

    Ok(())
}
```

### Query Preferences

```rust
use ffcv::{parse_prefs_js, query_preferences};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string("prefs.js")?;
    let entries = parse_prefs_js(&content)?;

    // Query network-related preferences
    let network_prefs = query_preferences(&entries, &["network.*"])?;

    // Display matching entries
    for entry in network_prefs {
        println!("{} = {:?}", entry.key, entry.value);
    }

    Ok(())
}
```

### Profile Discovery

```rust
use ffcv::list_profiles;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profiles = list_profiles(None)?;

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
use ffcv::{PrefEntry, PrefValue, PrefValueExt};

fn process_entry(entry: PrefEntry) {
    // Use convenience methods from PrefValueExt trait
    if let Some(enabled) = entry.value.as_bool() {
        if enabled {
            println!("{} is enabled", entry.key);
        } else {
            println!("{} is disabled", entry.key);
        }
    } else if let Some(n) = entry.value.as_int() {
        println!("{} = {} (integer)", entry.key, n);
    } else if let Some(s) = entry.value.as_string() {
        println!("{} = \"{}\"", entry.key, s);
    } else {
        println!("{} = {:?}", entry.key, entry.value);
    }
}
```

### Finding a Specific Profile

```rust
use ffcv::find_profile_path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Find a specific Firefox profile by name
    let profile_path = find_profile_path("default", None)?;

    println!("Profile path: {}", profile_path.display());

    Ok(())
}
```

## API Design

ffcv provides a clean, simplified API with all public types and functions available at the crate root:

**Core Types:**
- `PrefEntry` - A single preference entry with key, value, and type
- `PrefType` - The type of preference (User, Default, Locked, Sticky)
- `PrefValue` - The value of a preference (Bool, Int, Float, String, Null)
- `PrefValueExt` - Convenience trait for type-safe value access

**Core Functions:**
- `parse_prefs_js()` - Parse preference file contents
- `parse_prefs_js_file()` - Parse directly from a file path
- `query_preferences()` - Filter preferences by glob patterns
- `list_profiles()` - List all Firefox profiles
- `find_profile_path()` - Find a specific profile by name
- `get_prefs_path()` - Get the prefs.js path for a profile

All functions return `Result<T, Error>` for proper error handling.

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

## Version 1.0

ffcv provides a stable and well-tested API. The library offers a clean, simplified interface with comprehensive Firefox preference parsing capabilities. All public types and functions are available at the crate root for easy importing.

## Acknowledgments

Built for the Rust community to make Firefox configuration management easier and more programmatic.

Inspired by the need for better tools to understand and manage Firefox's extensive preference system.
