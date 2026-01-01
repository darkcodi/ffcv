//! Parser for Firefox preference files
//!
//! This module provides a recursive descent parser that converts tokenized
//! Firefox prefs.js files into a HashMap of preference values.

use crate::lexer::{Lexer, Token};
use crate::types::{PrefEntry, PrefType};
use std::collections::HashMap;

/// Parse the prefs.js file and extract all preferences
pub fn parse_prefs_js(content: &str) -> Result<HashMap<String, serde_json::Value>, anyhow::Error> {
    let mut parser = Parser::new(content);
    parser.parse()
}

/// Parse the prefs.js file and extract all preferences with their types
pub fn parse_prefs_js_with_types(
    content: &str,
) -> Result<HashMap<String, PrefEntry>, anyhow::Error> {
    let mut parser = Parser::new(content);
    parser.parse_with_types()
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
            Err(e) => {
                return Parser {
                    lexer,
                    current: None,
                    current_line: e.line,
                    current_column: e.column,
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

    /// Parse the entire input into a HashMap of preferences
    fn parse(&mut self) -> Result<HashMap<String, serde_json::Value>, anyhow::Error> {
        let mut preferences = HashMap::new();

        loop {
            match &self.current {
                None => {
                    // Error occurred during lexing
                    return Err(anyhow::anyhow!(
                        "Parse error at line {}:{}: Lexer error",
                        self.current_line,
                        self.current_column
                    ));
                }
                Some(Token::Eof) => break,
                Some(_) => {
                    let (key, value) = self.parse_statement()?;
                    preferences.insert(key, value);
                }
            }
        }

        Ok(preferences)
    }

    /// Parse the entire input into a HashMap of preferences with their types
    fn parse_with_types(&mut self) -> Result<HashMap<String, PrefEntry>, anyhow::Error> {
        let mut preferences = HashMap::new();

        loop {
            match &self.current {
                None => {
                    // Error occurred during lexing
                    return Err(anyhow::anyhow!(
                        "Parse error at line {}:{}: Lexer error",
                        self.current_line,
                        self.current_column
                    ));
                }
                Some(Token::Eof) => break,
                Some(_) => {
                    let (key, value, pref_type) = self.parse_statement_with_type()?;
                    preferences.insert(key.clone(), PrefEntry { value, pref_type });
                }
            }
        }

        Ok(preferences)
    }

    /// Parse a single statement: pref_type "(" key "," value ")" ";"
    fn parse_statement(&mut self) -> Result<(String, serde_json::Value), anyhow::Error> {
        // Parse the pref function name (user_pref, pref, lock_pref, sticky_pref)
        // We don't actually need to store the type since they all have the same syntax
        match &self.current {
            Some(Token::Identifier(_)) => {
                // Consume the identifier
                self.advance();
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Parse error at line {}:{}: Expected pref function name (user_pref, pref, lock_pref, sticky_pref), got {:?}",
                    self.current_line,
                    self.current_column,
                    self.current
                ));
            }
        }

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

        Ok((key, value))
    }

    /// Parse a single statement with type information: pref_type "(" key "," value ")" ";"
    fn parse_statement_with_type(
        &mut self,
    ) -> Result<(String, serde_json::Value, PrefType), anyhow::Error> {
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
    fn parse_pref_type_identifier(&mut self) -> Result<PrefType, anyhow::Error> {
        match &self.current {
            Some(Token::Identifier(ident)) => {
                let pref_type = match ident.as_str() {
                    "user_pref" => PrefType::User,
                    "pref" => PrefType::Default,
                    "lock_pref" => PrefType::Locked,
                    "sticky_pref" => PrefType::Sticky,
                    _ => {
                        return Err(anyhow::anyhow!(
                            "Parse error at line {}:{}: Unknown pref function '{}'. Expected user_pref, pref, lock_pref, or sticky_pref",
                            self.current_line,
                            self.current_column,
                            ident
                        ));
                    }
                };
                // Consume the identifier
                self.advance();
                Ok(pref_type)
            }
            _ => Err(anyhow::anyhow!(
                "Parse error at line {}:{}: Expected pref function name (user_pref, pref, lock_pref, sticky_pref), got {:?}",
                self.current_line,
                self.current_column,
                self.current
            )),
        }
    }

    /// Parse a value (string, number, boolean, null)
    fn parse_value(&mut self) -> Result<serde_json::Value, anyhow::Error> {
        match &self.current {
            Some(Token::String(s)) => {
                let result = serde_json::Value::String(s.clone());
                self.current.take();
                self.advance();
                Ok(result)
            }
            Some(Token::Number(n)) => {
                let num_value = *n;
                if let Some(num) = serde_json::Number::from_f64(num_value) {
                    let result = serde_json::Value::Number(num);
                    self.current.take();
                    self.advance();
                    Ok(result)
                } else {
                    Err(anyhow::anyhow!(
                        "Parse error at line {}:{}: Invalid number {}",
                        self.current_line,
                        self.current_column,
                        num_value
                    ))
                }
            }
            Some(Token::Boolean(b)) => {
                let result = serde_json::Value::Bool(*b);
                self.current.take();
                self.advance();
                Ok(result)
            }
            Some(Token::Null) => {
                self.current.take();
                self.advance();
                Ok(serde_json::Value::Null)
            }
            Some(token) => Err(anyhow::anyhow!(
                "Parse error at line {}:{}: Expected value, got {:?}",
                self.current_line,
                self.current_column,
                token
            )),
            None => Err(anyhow::anyhow!(
                "Parse error at line {}:{}: Unexpected end of input",
                self.current_line,
                self.current_column
            )),
        }
    }

    /// Expect a specific token and consume it
    fn expect_token(&mut self, expected: Token) -> Result<(), anyhow::Error> {
        match &self.current {
            Some(token) if *token == expected => {
                // Take the token and advance to next
                self.current.take();
                self.advance();
                Ok(())
            }
            Some(token) => Err(anyhow::anyhow!(
                "Parse error at line {}:{}: Expected {:?}, got {:?}",
                self.current_line,
                self.current_column,
                expected,
                token
            )),
            None => Err(anyhow::anyhow!(
                "Parse error at line {}:{}: Unexpected end of input",
                self.current_line,
                self.current_column
            )),
        }
    }

    /// Expect a string token and return its value
    fn expect_string(&mut self) -> Result<String, anyhow::Error> {
        match &self.current {
            Some(Token::String(s)) => {
                // Clone the string value before consuming the token
                let result = s.clone();
                self.current.take();
                self.advance();
                Ok(result)
            }
            Some(token) => Err(anyhow::anyhow!(
                "Parse error at line {}:{}: Expected string, got {:?}",
                self.current_line,
                self.current_column,
                token
            )),
            None => Err(anyhow::anyhow!(
                "Parse error at line {}:{}: Unexpected end of input",
                self.current_line,
                self.current_column
            )),
        }
    }

    /// Advance to the next token
    fn advance(&mut self) {
        self.current = match self.lexer.next_token() {
            Ok(token) => Some(token),
            Err(e) => {
                self.current_line = e.line;
                self.current_column = e.column;
                None
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_prefs_js_string() {
        let input = r#"user_pref("browser.startup.homepage", "https://example.com");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["browser.startup.homepage"],
            serde_json::Value::String("https://example.com".to_string())
        );
    }

    #[test]
    fn test_parse_prefs_js_boolean() {
        let input = r#"user_pref("javascript.enabled", true);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result["javascript.enabled"], serde_json::Value::Bool(true));
    }

    #[test]
    fn test_parse_prefs_js_integer() {
        let input = r#"user_pref("network.cookie.cookieBehavior", 0);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        // Check that the value is a number and equals 0
        match &result["network.cookie.cookieBehavior"] {
            serde_json::Value::Number(n) => {
                assert_eq!(n.as_f64(), Some(0.0));
            }
            _ => panic!("Expected number"),
        }
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
        assert_eq!(
            result["test"],
            serde_json::Value::String("value with \"quotes\"".to_string())
        );
    }

    #[test]
    fn test_parse_value_float() {
        let input = r#"user_pref("test", 3.14);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result["test"].is_number());
    }

    #[test]
    fn test_parse_value_null() {
        let input = r#"user_pref("test", null);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result["test"], serde_json::Value::Null);
    }

    // New tests for enhanced functionality

    #[test]
    fn test_parse_default_pref() {
        let input = r#"pref("browser.startup.homepage", "https://example.com");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["browser.startup.homepage"],
            serde_json::Value::String("https://example.com".to_string())
        );
    }

    #[test]
    fn test_parse_locked_pref() {
        let input = r#"lock_pref("javascript.enabled", false);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result["javascript.enabled"], serde_json::Value::Bool(false));
    }

    #[test]
    fn test_parse_sticky_pref() {
        let input = r#"sticky_pref("network.proxy.type", 1);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        // Check that the value is a number and equals 1
        match &result["network.proxy.type"] {
            serde_json::Value::Number(n) => {
                assert_eq!(n.as_f64(), Some(1.0));
            }
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_parse_complex_url_with_commas() {
        let input = r#"user_pref("complex.url", "http://example.com?foo=bar,baz");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["complex.url"],
            serde_json::Value::String("http://example.com?foo=bar,baz".to_string())
        );
    }

    #[test]
    fn test_parse_uuid() {
        let input = r#"user_pref("test.id", "c0ffeec0-ffee-c0ff-eec0-ffeec0ffeec0");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["test.id"],
            serde_json::Value::String("c0ffeec0-ffee-c0ff-eec0-ffeec0ffeec0".to_string())
        );
    }

    #[test]
    fn test_parse_json_object_string() {
        let input = r#"user_pref("test", "{\"key\":\"value\"}");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["test"],
            serde_json::Value::String("{\"key\":\"value\"}".to_string())
        );
    }

    #[test]
    fn test_parse_json_array_string() {
        let input = r#"user_pref("test", "[1,2,3]");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["test"],
            serde_json::Value::String("[1,2,3]".to_string())
        );
    }

    #[test]
    fn test_parse_backslash_escapes() {
        let input = r#"user_pref("test", "C:\\path\\to\\file");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        // Note: lexer processes the escapes, so we get single backslashes
        assert_eq!(
            result["test"],
            serde_json::Value::String("C:\\path\\to\\file".to_string())
        );
    }

    #[test]
    fn test_parse_unicode_escape() {
        let input = r#"user_pref("test", "\u0041");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result["test"], serde_json::Value::String("A".to_string()));
    }

    #[test]
    fn test_parse_hex_escape() {
        let input = r#"user_pref("test", "\x41");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result["test"], serde_json::Value::String("A".to_string()));
    }

    #[test]
    fn test_parse_newline_tab_escapes() {
        let input = r#"user_pref("test", "line1\nline2\ttab");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["test"],
            serde_json::Value::String("line1\nline2\ttab".to_string())
        );
    }

    #[test]
    fn test_parse_negative_integer() {
        let input = r#"user_pref("test", -42);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        // Check that the value is a number and equals -42
        match &result["test"] {
            serde_json::Value::Number(n) => {
                assert_eq!(n.as_f64(), Some(-42.0));
            }
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_parse_negative_float() {
        let input = r#"user_pref("test", -3.14);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result["test"].is_number());
    }

    #[test]
    fn test_parse_scientific_notation_positive() {
        let input = r#"user_pref("test", 1.5e10);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result["test"].is_number());
    }

    #[test]
    fn test_parse_scientific_notation_negative() {
        let input = r#"user_pref("test", 3e-8);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result["test"].is_number());
    }

    #[test]
    fn test_parse_scientific_notation_decimal() {
        let input = r#"user_pref("test", -2.5e+3);"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result["test"].is_number());
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
        // This should actually work since we accept any identifier
        // The old implementation would also accept it
        assert!(result.is_ok());
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
            result["test"],
            serde_json::Value::String("value".to_string())
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
        assert_eq!(
            result["sidebar.backupState"],
            serde_json::Value::String("{\"command\":\"\",\"panelOpen\":false}".to_string())
        );
    }

    #[test]
    fn test_firefox_page_actions() {
        let input =
            r#"user_pref("browser.pageActions.persistedActions", "{\"ids\":[\"bookmark\"]}");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["browser.pageActions.persistedActions"],
            serde_json::Value::String("{\"ids\":[\"bookmark\"]}".to_string())
        );
    }

    #[test]
    fn test_firefox_telemetry_id() {
        let input = r#"user_pref("toolkit.telemetry.cachedClientID", "c0ffeec0-ffee-c0ff-eec0-ffeec0ffeec0");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["toolkit.telemetry.cachedClientID"],
            serde_json::Value::String("c0ffeec0-ffee-c0ff-eec0-ffeec0ffeec0".to_string())
        );
    }

    #[test]
    fn test_parse_backspace_escape() {
        let input = r#"user_pref("test", "value\bwith\bbackspace");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["test"],
            serde_json::Value::String("value\x08with\x08backspace".to_string())
        );
    }

    #[test]
    fn test_parse_form_feed_escape() {
        let input = r#"user_pref("test", "value\fform\ffeed");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["test"],
            serde_json::Value::String("value\x0cform\x0cfeed".to_string())
        );
    }

    #[test]
    fn test_parse_null_escape() {
        let input = r#"user_pref("test", "null\0character");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["test"],
            serde_json::Value::String("null\x00character".to_string())
        );
    }

    #[test]
    fn test_parse_all_new_escapes() {
        let input = r#"user_pref("test", "\b\f\0");"#;
        let result = parse_prefs_js(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["test"],
            serde_json::Value::String("\x08\x0c\x00".to_string())
        );
    }

    #[test]
    fn test_parse_octal_escape_error() {
        let input = r#"user_pref("test", "\00");"#;
        let result = parse_prefs_js(input);
        assert!(result.is_err());
    }

    // Tests for parse_prefs_js_with_types

    #[test]
    fn test_parse_user_pref_with_type() {
        let input = r#"user_pref("browser.startup.homepage", "https://example.com");"#;
        let result = parse_prefs_js_with_types(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["browser.startup.homepage"].pref_type,
            crate::types::PrefType::User
        );
        assert_eq!(
            result["browser.startup.homepage"].value,
            serde_json::Value::String("https://example.com".to_string())
        );
    }

    #[test]
    fn test_parse_default_pref_with_type() {
        let input = r#"pref("javascript.enabled", true);"#;
        let result = parse_prefs_js_with_types(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["javascript.enabled"].pref_type,
            crate::types::PrefType::Default
        );
        assert_eq!(
            result["javascript.enabled"].value,
            serde_json::Value::Bool(true)
        );
    }

    #[test]
    fn test_parse_locked_pref_with_type() {
        let input = r#"lock_pref("network.proxy.type", 1);"#;
        let result = parse_prefs_js_with_types(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["network.proxy.type"].pref_type,
            crate::types::PrefType::Locked
        );
        match &result["network.proxy.type"].value {
            serde_json::Value::Number(n) => {
                assert_eq!(n.as_f64(), Some(1.0));
            }
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_parse_sticky_pref_with_type() {
        let input = r#"sticky_pref("test.pref", "sticky value");"#;
        let result = parse_prefs_js_with_types(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result["test.pref"].pref_type,
            crate::types::PrefType::Sticky
        );
        assert_eq!(
            result["test.pref"].value,
            serde_json::Value::String("sticky value".to_string())
        );
    }

    #[test]
    fn test_parse_mixed_pref_types() {
        let input = r#"
            user_pref("user.pref", "value1");
            pref("default.pref", "value2");
            lock_pref("locked.pref", "value3");
            sticky_pref("sticky.pref", "value4");
        "#;
        let result = parse_prefs_js_with_types(input).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result["user.pref"].pref_type, crate::types::PrefType::User);
        assert_eq!(
            result["default.pref"].pref_type,
            crate::types::PrefType::Default
        );
        assert_eq!(
            result["locked.pref"].pref_type,
            crate::types::PrefType::Locked
        );
        assert_eq!(
            result["sticky.pref"].pref_type,
            crate::types::PrefType::Sticky
        );
    }

    #[test]
    fn test_parse_unknown_pref_function_error() {
        let input = r#"unknown_pref("test", "value");"#;
        let result = parse_prefs_js_with_types(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unknown pref function"));
    }
}
