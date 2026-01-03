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
