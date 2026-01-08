//! Parser for Firefox preference files (prefs.js)
//!
//! This module provides a recursive descent parser that converts Firefox's
//! custom JavaScript-like preference syntax into structured data.
//!
//! # Format
//!
//! Firefox preferences are stored using a JavaScript-like syntax:
//!
//! ```text
//! user_pref("preference.name", value);
//! pref("preference.name", value);           // default
//! lock_pref("preference.name", value);      // locked
//! sticky_pref("preference.name", value);    // sticky
//! ```
//!
//! # Example
//!
//! ```rust
//! use ffcv::{parse_prefs_js, PrefType, PrefValue};
//!
//! let content = r#"
//!     // This is a comment
//!     user_pref("browser.startup.homepage", "https://example.com");
//!     pref("javascript.enabled", true);
//!     user_pref("network.proxy.port", 8080);
//! "#;
//!
//! let prefs = parse_prefs_js(content)?;
//! let homepage = prefs.iter().find(|e| e.key == "browser.startup.homepage").unwrap();
//! assert_eq!(homepage.value, PrefValue::String("https://example.com".to_string()));
//! assert_eq!(homepage.pref_type, PrefType::User);
//! # Ok::<(), ffcv::Error>(())
//! ```

use crate::error::{Error, Result};
use crate::lexer::{Lexer, Token};
use crate::types::{PrefEntry, PrefSource, PrefType, PrefValue};

/// Parse the prefs.js file and extract all preferences
///
/// This function parses Firefox preference files and returns a Vec of
/// preference entries with their types and values.
///
/// # Example
///
/// ```rust
/// use ffcv::{parse_prefs_js, PrefEntry, PrefType, PrefValue};
///
/// let content = r#"
///     user_pref("browser.startup.homepage", "https://example.com");
///     user_pref("javascript.enabled", true);
/// "#;
///
/// let prefs = parse_prefs_js(content)?;
/// let homepage = prefs.iter().find(|e| e.key == "browser.startup.homepage").unwrap();
/// assert_eq!(homepage.value, PrefValue::String("https://example.com".to_string()));
/// assert_eq!(homepage.pref_type, PrefType::User);
/// # Ok::<(), ffcv::Error>(())
/// ```
pub fn parse_prefs_js(content: &str) -> Result<Vec<PrefEntry>> {
    let mut parser = Parser::new(content);
    parser.parse()
}

/// Parse a prefs.js file directly from a file path
///
/// This is a convenience function that reads the file and parses it in one step.
pub fn parse_prefs_js_file(path: &std::path::Path) -> Result<Vec<PrefEntry>> {
    let content = std::fs::read_to_string(path)?;
    parse_prefs_js(&content)
}

/// Parser for Firefox preference files
struct Parser<'a> {
    lexer: Lexer<'a>,
    /// Current lookahead token
    current: Option<Token>,
    /// Current line for error reporting
    current_line: usize,
    /// Current column for error reporting
    current_column: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given input
    fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        // Prime the pump by getting the first token
        let current = match lexer.next_token() {
            Ok(token) => Some(token),
            Err(Error::Lexer { line, column, .. }) => {
                return Parser {
                    lexer,
                    current: None,
                    current_line: line,
                    current_column: column,
                }
            }
            // This shouldn't happen since lexer only returns Error::Lexer
            Err(_) => {
                return Parser {
                    lexer,
                    current: None,
                    current_line: 1,
                    current_column: 1,
                }
            }
        };

        Parser {
            lexer,
            current,
            current_line: 1,
            current_column: 1,
        }
    }

    /// Parse the entire input into a Vec of preferences with their types
    fn parse(&mut self) -> Result<Vec<PrefEntry>> {
        let mut preferences = Vec::new();

        loop {
            match &self.current {
                None => {
                    // Error occurred during lexing
                    return Err(Error::Parser {
                        line: self.current_line,
                        column: self.current_column,
                        message: "Lexer error".to_string(),
                    });
                }
                Some(Token::Eof) => break,
                Some(_) => {
                    let (key, value, pref_type) = self.parse_statement_with_type()?;
                    let explanation = crate::explanations::get_preference_explanation_static(&key);
                    preferences.push(PrefEntry {
                        key,
                        value,
                        pref_type,
                        explanation,
                        source: Some(PrefSource::User),
                        source_file: Some("prefs.js".to_string()),
                    });
                }
            }
        }

        Ok(preferences)
    }

    /// Parse a single statement with type information: pref_type "(" key "," value ")" ";"
    fn parse_statement_with_type(&mut self) -> Result<(String, PrefValue, PrefType)> {
        // Parse and capture the pref function name (user_pref, pref, lock_pref, sticky_pref)
        let pref_type = self.parse_pref_type_identifier()?;

        // Expect left parenthesis
        self.expect_token(Token::LeftParen)?;

        // Expect key (string)
        let key = self.expect_string()?;

        // Expect comma
        self.expect_token(Token::Comma)?;

        // Parse value
        let value = self.parse_value()?;

        // Expect right parenthesis
        self.expect_token(Token::RightParen)?;

        // Expect semicolon
        self.expect_token(Token::Semicolon)?;

        Ok((key, value, pref_type))
    }

    /// Parse the pref type identifier and return the corresponding PrefType
    fn parse_pref_type_identifier(&mut self) -> Result<PrefType> {
        match &self.current {
            Some(Token::Identifier(ident)) => {
                let pref_type = match ident.as_str() {
                    "user_pref" => PrefType::User,
                    "pref" => PrefType::Default,
                    "lock_pref" => PrefType::Locked,
                    "sticky_pref" => PrefType::Sticky,
                    _ => {
                        return Err(Error::Parser {
                            line: self.current_line,
                            column: self.current_column,
                            message: format!(
                                "Unknown pref function '{}'. Expected user_pref, pref, lock_pref, or sticky_pref",
                                ident
                            ),
                        });
                    }
                };
                // Consume the identifier
                self.advance();
                Ok(pref_type)
            }
            _ => Err(Error::Parser {
                line: self.current_line,
                column: self.current_column,
                message: format!(
                    "Expected pref function name (user_pref, pref, lock_pref, sticky_pref), got {:?}",
                    self.current
                ),
            }),
        }
    }

    /// Parse a value (string, number, boolean, null)
    fn parse_value(&mut self) -> Result<PrefValue> {
        match &self.current {
            Some(Token::String(_)) => {
                let token = std::mem::take(&mut self.current);
                self.advance();
                match token {
                    Some(Token::String(s)) => Ok(PrefValue::String(s)),
                    _ => unreachable!(),
                }
            }
            Some(Token::Number(n)) => {
                let num_value = *n;
                self.current.take();
                self.advance();
                Ok(PrefValue::from_f64(num_value))
            }
            Some(Token::Boolean(b)) => {
                let result = PrefValue::Bool(*b);
                self.current.take();
                self.advance();
                Ok(result)
            }
            Some(Token::Null) => {
                self.current.take();
                self.advance();
                Ok(PrefValue::Null)
            }
            Some(token) => Err(Error::Parser {
                line: self.current_line,
                column: self.current_column,
                message: format!("Expected value, got {:?}", token),
            }),
            None => Err(Error::Parser {
                line: self.current_line,
                column: self.current_column,
                message: "Unexpected end of input".to_string(),
            }),
        }
    }

    /// Expect a specific token and consume it
    fn expect_token(&mut self, expected: Token) -> Result<()> {
        match &self.current {
            Some(token) if *token == expected => {
                // Take the token and advance to next
                self.current.take();
                self.advance();
                Ok(())
            }
            Some(token) => Err(Error::Parser {
                line: self.current_line,
                column: self.current_column,
                message: format!("Expected {:?}, got {:?}", expected, token),
            }),
            None => Err(Error::Parser {
                line: self.current_line,
                column: self.current_column,
                message: "Unexpected end of input".to_string(),
            }),
        }
    }

    /// Expect a string token and return its value
    fn expect_string(&mut self) -> Result<String> {
        match &self.current {
            Some(Token::String(_)) => {
                // Take the token to extract the String without cloning
                let token = std::mem::take(&mut self.current);
                self.advance();
                match token {
                    Some(Token::String(s)) => Ok(s),
                    _ => unreachable!(),
                }
            }
            Some(token) => Err(Error::Parser {
                line: self.current_line,
                column: self.current_column,
                message: format!("Expected string, got {:?}", token),
            }),
            None => Err(Error::Parser {
                line: self.current_line,
                column: self.current_column,
                message: "Unexpected end of input".to_string(),
            }),
        }
    }

    /// Advance to the next token
    fn advance(&mut self) {
        self.current = match self.lexer.next_token() {
            Ok(token) => Some(token),
            Err(Error::Lexer { line, column, .. }) => {
                self.current_line = line;
                self.current_column = column;
                None
            }
            Err(_) => {
                // Lexer should only return Lexer errors, but handle other errors gracefully
                self.current = None;
                None
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PrefValueExt;

    #[test]
    fn test_parse_prefs_js_string() {
        let input = r#"user_pref("browser.startup.homepage", "https://example.com");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result
            .iter()
            .find(|e| e.key == "browser.startup.homepage")
            .unwrap();
        assert_eq!(
            entry.value,
            PrefValue::String("https://example.com".to_string())
        );
        assert_eq!(entry.pref_type, PrefType::User);
    }

    #[test]
    fn test_parse_prefs_js_boolean() {
        let input = r#"user_pref("javascript.enabled", true);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result
            .iter()
            .find(|e| e.key == "javascript.enabled")
            .unwrap();
        assert_eq!(entry.value, PrefValue::Bool(true));
        assert_eq!(entry.pref_type, PrefType::User);
    }

    #[test]
    fn test_parse_prefs_js_integer() {
        let input = r#"user_pref("network.cookie.cookieBehavior", 0);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result
            .iter()
            .find(|e| e.key == "network.cookie.cookieBehavior")
            .unwrap();
        // Check that the value is a number and equals 0
        match &entry.value {
            PrefValue::Integer(n) => {
                assert_eq!(*n, 0.0 as i64);
            }
            _ => panic!("Expected number"),
        }
        assert_eq!(entry.pref_type, PrefType::User);
    }

    #[test]
    fn test_parse_prefs_js_multiple() {
        let input = r#"
            user_pref("browser.startup.homepage", "https://example.com");
            user_pref("javascript.enabled", true);
            user_pref("network.cookie.cookieBehavior", 0);
        "#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_parse_prefs_js_with_comments() {
        let input = r#"
            // This is a comment
            user_pref("browser.startup.homepage", "https://example.com");
            // Another comment
            user_pref("javascript.enabled", true);
        "#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_value_string_with_escaped_quotes() {
        let input = r#"user_pref("test", "value with \"quotes\"");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result.iter().find(|e| e.key == "test").unwrap();
        assert_eq!(
            entry.value,
            PrefValue::String("value with \"quotes\"".to_string())
        );
        assert_eq!(entry.pref_type, PrefType::User);
    }

    #[test]
    fn test_parse_value_float() {
        let input = r#"user_pref("test", 3.14);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result.iter().find(|e| e.key == "test").unwrap();
        assert!(entry.value.is_number());
        assert_eq!(entry.pref_type, PrefType::User);
    }

    #[test]
    fn test_parse_value_null() {
        let input = r#"user_pref("test", null);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result.iter().find(|e| e.key == "test").unwrap();
        assert_eq!(entry.value, PrefValue::Null);
        assert_eq!(entry.pref_type, PrefType::User);
    }

    // New tests for enhanced functionality

    #[test]
    fn test_parse_default_pref() {
        let input = r#"pref("browser.startup.homepage", "https://example.com");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result
            .iter()
            .find(|e| e.key == "browser.startup.homepage")
            .unwrap();
        assert_eq!(
            entry.value,
            PrefValue::String("https://example.com".to_string())
        );
        assert_eq!(entry.pref_type, PrefType::Default);
    }

    #[test]
    fn test_parse_locked_pref() {
        let input = r#"lock_pref("javascript.enabled", false);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result
            .iter()
            .find(|e| e.key == "javascript.enabled")
            .unwrap();
        assert_eq!(entry.value, PrefValue::Bool(false));
        assert_eq!(entry.pref_type, PrefType::Locked);
    }

    #[test]
    fn test_parse_sticky_pref() {
        let input = r#"sticky_pref("network.proxy.type", 1);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result
            .iter()
            .find(|e| e.key == "network.proxy.type")
            .unwrap();
        // Check that the value is a number and equals 1
        match &entry.value {
            PrefValue::Integer(n) => {
                assert_eq!(*n, 1.0 as i64);
            }
            _ => panic!("Expected number"),
        }
        assert_eq!(entry.pref_type, PrefType::Sticky);
    }

    #[test]
    fn test_parse_complex_url_with_commas() {
        let input = r#"user_pref("complex.url", "http://example.com?foo=bar,baz");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result.iter().find(|e| e.key == "complex.url").unwrap();
        assert_eq!(
            entry.value,
            PrefValue::String("http://example.com?foo=bar,baz".to_string())
        );
        assert_eq!(entry.pref_type, PrefType::User);
    }

    #[test]
    fn test_parse_uuid() {
        let input = r#"user_pref("test.id", "c0ffeec0-ffee-c0ff-eec0-ffeec0ffeec0");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result.iter().find(|e| e.key == "test.id").unwrap();
        assert_eq!(
            entry.value,
            PrefValue::String("c0ffeec0-ffee-c0ff-eec0-ffeec0ffeec0".to_string())
        );
        assert_eq!(entry.pref_type, PrefType::User);
    }

    #[test]
    fn test_parse_json_object_string() {
        let input = r#"user_pref("test", "{\"key\":\"value\"}");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.iter().find(|e| e.key == "test").unwrap().value,
            PrefValue::String("{\"key\":\"value\"}".to_string())
        );
    }

    #[test]
    fn test_parse_json_array_string() {
        let input = r#"user_pref("test", "[1,2,3]");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.iter().find(|e| e.key == "test").unwrap().value,
            PrefValue::String("[1,2,3]".to_string())
        );
    }

    #[test]
    fn test_parse_backslash_escapes() {
        let input = r#"user_pref("test", "C:\\path\\to\\file");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        // Note: lexer processes the escapes, so we get single backslashes
        assert_eq!(
            result.iter().find(|e| e.key == "test").unwrap().value,
            PrefValue::String("C:\\path\\to\\file".to_string())
        );
    }

    #[test]
    fn test_parse_unicode_escape() {
        let input = r#"user_pref("test", "\u0041");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.iter().find(|e| e.key == "test").unwrap().value,
            PrefValue::String("A".to_string())
        );
    }

    #[test]
    fn test_parse_hex_escape() {
        let input = r#"user_pref("test", "\x41");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.iter().find(|e| e.key == "test").unwrap().value,
            PrefValue::String("A".to_string())
        );
    }

    #[test]
    fn test_parse_newline_tab_escapes() {
        let input = r#"user_pref("test", "line1\nline2\ttab");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.iter().find(|e| e.key == "test").unwrap().value,
            PrefValue::String("line1\nline2\ttab".to_string())
        );
    }

    #[test]
    fn test_parse_negative_integer() {
        let input = r#"user_pref("test", -42);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        // Check that the value is a number and equals -42
        match &result.iter().find(|e| e.key == "test").unwrap().value {
            PrefValue::Integer(n) => {
                assert_eq!(*n, -42.0 as i64);
            }
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_parse_negative_float() {
        let input = r#"user_pref("test", -3.14);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result
            .iter()
            .find(|e| e.key == "test")
            .unwrap()
            .value
            .is_number());
    }

    #[test]
    fn test_parse_scientific_notation_positive() {
        let input = r#"user_pref("test", 1.5e10);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result
            .iter()
            .find(|e| e.key == "test")
            .unwrap()
            .value
            .is_number());
    }

    #[test]
    fn test_parse_scientific_notation_negative() {
        let input = r#"user_pref("test", 3e-8);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result
            .iter()
            .find(|e| e.key == "test")
            .unwrap()
            .value
            .is_number());
    }

    #[test]
    fn test_parse_scientific_notation_decimal() {
        let input = r#"user_pref("test", -2.5e+3);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result
            .iter()
            .find(|e| e.key == "test")
            .unwrap()
            .value
            .is_number());
    }

    #[test]
    fn test_malformed_missing_semicolon() {
        let input = r#"user_pref("test", "value")"#;
        let result = parse_prefs_js(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_unclosed_string() {
        let input = r#"user_pref("test", "value);"#;
        let result = parse_prefs_js(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_invalid_escape() {
        let input = r#"user_pref("test", "\xGG");"#;
        let result = parse_prefs_js(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_unknown_pref_function() {
        let input = r#"unknown_func("test", "value");"#;
        let result = parse_prefs_js(input);
        // Now that we parse types, we reject unknown pref functions
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown pref function"));
    }

    #[test]
    fn test_multiline_statement() {
        let input = r#"user_pref(
            "test",
            "value"
        );"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.iter().find(|e| e.key == "test").unwrap().value,
            PrefValue::String("value".to_string())
        );
    }

    #[test]
    fn test_multiline_with_comments() {
        let input = r#"
            user_pref(
                "test",  // inline comment
                "value"
            );
        "#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_block_comment_in_statement() {
        let input = r#"user_pref(/* comment */ "test", "value");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_multiple_line_comments() {
        let input = r#"
            // Comment 1
            user_pref("test1", "value1");
            // Comment 2
            user_pref("test2", "value2");
        "#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_firefox_sidebar_state() {
        let input =
            r#"user_pref("sidebar.backupState", "{\"command\":\"\",\"panelOpen\":false}");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result
            .iter()
            .find(|e| e.key == "sidebar.backupState")
            .unwrap();
        assert_eq!(
            entry.value,
            PrefValue::String("{\"command\":\"\",\"panelOpen\":false}".to_string())
        );
        assert_eq!(entry.pref_type, PrefType::User);
    }

    #[test]
    fn test_firefox_page_actions() {
        let input =
            r#"user_pref("browser.pageActions.persistedActions", "{\"ids\":[\"bookmark\"]}");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result
            .iter()
            .find(|e| e.key == "browser.pageActions.persistedActions")
            .unwrap();
        assert_eq!(
            entry.value,
            PrefValue::String("{\"ids\":[\"bookmark\"]}".to_string())
        );
        assert_eq!(entry.pref_type, PrefType::User);
    }

    #[test]
    fn test_firefox_telemetry_id() {
        let input = r#"user_pref("toolkit.telemetry.cachedClientID", "c0ffeec0-ffee-c0ff-eec0-ffeec0ffeec0");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result
            .iter()
            .find(|e| e.key == "toolkit.telemetry.cachedClientID")
            .unwrap();
        assert_eq!(
            entry.value,
            PrefValue::String("c0ffeec0-ffee-c0ff-eec0-ffeec0ffeec0".to_string())
        );
        assert_eq!(entry.pref_type, PrefType::User);
    }

    #[test]
    fn test_parse_backspace_escape() {
        let input = r#"user_pref("test", "value\bwith\bbackspace");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.iter().find(|e| e.key == "test").unwrap().value,
            PrefValue::String("value\x08with\x08backspace".to_string())
        );
    }

    #[test]
    fn test_parse_form_feed_escape() {
        let input = r#"user_pref("test", "value\fform\ffeed");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.iter().find(|e| e.key == "test").unwrap().value,
            PrefValue::String("value\x0cform\x0cfeed".to_string())
        );
    }

    #[test]
    fn test_parse_null_escape() {
        let input = r#"user_pref("test", "null\0character");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.iter().find(|e| e.key == "test").unwrap().value,
            PrefValue::String("null\x00character".to_string())
        );
    }

    #[test]
    fn test_parse_all_new_escapes() {
        let input = r#"user_pref("test", "\b\f\0");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.iter().find(|e| e.key == "test").unwrap().value,
            PrefValue::String("\x08\x0c\x00".to_string())
        );
    }

    #[test]
    fn test_parse_octal_escape_error() {
        let input = r#"user_pref("test", "\00");"#;
        let result = parse_prefs_js(input);
        assert!(result.is_err());
    }

    // Tests for parse_prefs_js

    #[test]
    fn test_parse_user_pref_with_type() {
        let input = r#"user_pref("browser.startup.homepage", "https://example.com");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result
            .iter()
            .find(|e| e.key == "browser.startup.homepage")
            .unwrap();
        assert_eq!(entry.key, "browser.startup.homepage");
        assert_eq!(entry.pref_type, crate::types::PrefType::User);
        assert_eq!(
            entry.value,
            PrefValue::String("https://example.com".to_string())
        );
    }

    #[test]
    fn test_parse_default_pref_with_type() {
        let input = r#"pref("javascript.enabled", true);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result
            .iter()
            .find(|e| e.key == "javascript.enabled")
            .unwrap();
        assert_eq!(entry.key, "javascript.enabled");
        assert_eq!(entry.pref_type, crate::types::PrefType::Default);
        assert_eq!(entry.value, PrefValue::Bool(true));
    }

    #[test]
    fn test_parse_locked_pref_with_type() {
        let input = r#"lock_pref("network.proxy.type", 1);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result
            .iter()
            .find(|e| e.key == "network.proxy.type")
            .unwrap();
        assert_eq!(entry.key, "network.proxy.type");
        assert_eq!(entry.pref_type, crate::types::PrefType::Locked);
        match &entry.value {
            PrefValue::Integer(n) => {
                assert_eq!(*n, 1.0 as i64);
            }
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_parse_sticky_pref_with_type() {
        let input = r#"sticky_pref("test.pref", "sticky value");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        let entry = result.iter().find(|e| e.key == "test.pref").unwrap();
        assert_eq!(entry.key, "test.pref");
        assert_eq!(entry.pref_type, crate::types::PrefType::Sticky);
        assert_eq!(entry.value, PrefValue::String("sticky value".to_string()));
    }

    #[test]
    fn test_parse_mixed_pref_types() {
        let input = r#"
            user_pref("user.pref", "value1");
            pref("default.pref", "value2");
            lock_pref("locked.pref", "value3");
            sticky_pref("sticky.pref", "value4");
        "#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 4);

        let user_entry = result.iter().find(|e| e.key == "user.pref").unwrap();
        assert_eq!(user_entry.key, "user.pref");
        assert_eq!(user_entry.pref_type, crate::types::PrefType::User);

        let default_entry = result.iter().find(|e| e.key == "default.pref").unwrap();
        assert_eq!(default_entry.key, "default.pref");
        assert_eq!(default_entry.pref_type, crate::types::PrefType::Default);

        let locked_entry = result.iter().find(|e| e.key == "locked.pref").unwrap();
        assert_eq!(locked_entry.key, "locked.pref");
        assert_eq!(locked_entry.pref_type, crate::types::PrefType::Locked);

        let sticky_entry = result.iter().find(|e| e.key == "sticky.pref").unwrap();
        assert_eq!(sticky_entry.key, "sticky.pref");
        assert_eq!(sticky_entry.pref_type, crate::types::PrefType::Sticky);
    }

    #[test]
    fn test_parse_unknown_pref_function_error() {
        let input = r#"unknown_pref("test", "value");"#;
        let result = parse_prefs_js(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unknown pref function"));
    }

    #[test]
    fn test_parse_with_explanations() {
        let input = r#"
            user_pref("javascript.enabled", true);
            user_pref("unknown.preference", "test");
        "#;

        let result = parse_prefs_js(input).unwrap();

        // javascript.enabled has an explanation
        let js_entry = result
            .iter()
            .find(|e| e.key == "javascript.enabled")
            .unwrap();
        assert_eq!(js_entry.key, "javascript.enabled");
        assert_eq!(js_entry.pref_type, PrefType::User);
        assert!(js_entry.explanation.is_some());
        assert!(js_entry.explanation.unwrap().contains("JavaScript"));

        // unknown.preference has no explanation
        let unknown_entry = result
            .iter()
            .find(|e| e.key == "unknown.preference")
            .unwrap();
        assert_eq!(unknown_entry.key, "unknown.preference");
        assert!(unknown_entry.explanation.is_none());
    }
}
