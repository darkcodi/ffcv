//! Error types for Firefox preference parsing and operations
//!
//! This module defines the error types used throughout the ffcv library.
//! All public functions return [`Result<T, Error>`] for consistent error handling.

use std::path::PathBuf;

/// Errors that can occur during Firefox preference parsing and operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Lexer error during tokenization
    #[error("Lexer error at line {line}, column {column}: {message}")]
    Lexer {
        line: usize,
        column: usize,
        message: String,
    },

    /// Parser error during parsing
    #[error("Parser error at line {line}, column {column}: {message}")]
    Parser {
        line: usize,
        column: usize,
        message: String,
    },

    /// Invalid preference type or value
    #[error("Invalid preference: {0}")]
    InvalidPreference(String),

    /// I/O error during file operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Profile not found
    #[error("Profile '{name}' not found in {directory}")]
    ProfileNotFound { name: String, directory: PathBuf },

    /// Invalid profile directory
    #[error("Invalid profile directory: {0}")]
    InvalidProfileDirectory(PathBuf),

    /// Invalid glob pattern in query
    #[error("Invalid glob pattern: {0}")]
    InvalidGlobPattern(String),

    /// profiles.ini parsing error
    #[error("Failed to parse profiles.ini: {0}")]
    ProfilesIniParse(String),

    /// Firefox installation not found
    #[error("Firefox installation not found. Searched paths: {searched_paths}")]
    FirefoxNotFound { searched_paths: String },

    /// Error reading or parsing omni.ja file
    #[error("omni.ja error: {0}")]
    OmniJaError(String),

    /// Preference file not found
    #[error("Preference file not found: {file}")]
    PrefFileNotFound { file: String },

    /// File extraction failed
    #[error("File extraction failed: {0}")]
    ExtractionFailed(String),

    /// omni.ja file is too large to process safely
    #[error(
        "omni.ja file is too large ({actual} bytes). Maximum safe size is {limit} bytes. \
         You can increase this limit with --max-file-size if you're sure the file is valid."
    )]
    OmniJaTooLarge { actual: usize, limit: usize },
}

/// Result type alias for convenience
///
/// All public functions in the ffcv library return this type alias for
/// consistent error handling.
///
/// # Example
///
/// ```rust
/// use ffcv::{Result, parse_prefs_js};
///
/// fn parse_and_check(content: &str) -> Result<()> {
///     let config = parse_prefs_js(content)?;
///     // ... do something with config
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;
