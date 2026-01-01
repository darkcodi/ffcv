use clap::{Parser, Subcommand};
use std::str::FromStr;

/// View Firefox configuration from the command line
#[derive(Parser, Debug)]
#[command(name = "ffcv")]
#[command(about = "View Firefox configuration from the command line")]
#[command(
    long_about = "ffcv lets you view Firefox configuration (prefs.js) as JSON \
from the command line. Use subcommands to list profiles or inspect configuration."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all available Firefox profiles
    Profile {
        /// Path to Firefox profiles directory (overrides auto-detection)
        #[arg(short = 'd', long = "profiles-dir")]
        profiles_dir: Option<std::path::PathBuf>,
    },

    /// View Firefox configuration for a profile
    Config {
        /// Firefox profile name (default: "default")
        #[arg(short = 'p', long, default_value = "default")]
        profile: String,

        /// Read preferences from stdin instead of profile directory
        #[arg(long, conflicts_with = "profile")]
        stdin: bool,

        /// Path to Firefox profiles directory (overrides auto-detection)
        #[arg(short = 'd', long = "profiles-dir")]
        profiles_dir: Option<std::path::PathBuf>,

        /// Query preferences by glob pattern (e.g., "network.*", "browser.*.enabled")
        #[arg(long, conflicts_with = "get")]
        query: Vec<String>,

        /// Get a single preference by exact key name (raw output)
        #[arg(long, conflicts_with = "query")]
        get: Option<String>,

        /// Output format type (default: json-object)
        #[arg(
            long = "output-type",
            default_value = "json-object",
            conflicts_with = "get"
        )]
        output_type: OutputType,

        /// Show only preferences without explanations (hidden flag)
        #[arg(long = "unexplained-only", hide = true)]
        unexplained_only: bool,
    },
}

/// Output format type for configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    JsonObject,
    JsonArray,
}

impl FromStr for OutputType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json-object" => Ok(OutputType::JsonObject),
            "json-array" => Ok(OutputType::JsonArray),
            _ => Err(format!(
                "Invalid output type: '{}'. Valid values: json-object, json-array",
                s
            )),
        }
    }
}

impl std::fmt::Display for OutputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputType::JsonObject => write!(f, "json-object"),
            OutputType::JsonArray => write!(f, "json-array"),
        }
    }
}
