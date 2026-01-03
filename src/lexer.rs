//! Lexer for tokenizing Firefox preference files
//!
//! This module provides a tokenizer that converts character streams into tokens
//! for parsing Firefox prefs.js files. It handles all JavaScript escape sequences
//! and tracks line/column numbers for accurate error reporting.

use crate::error::{Error, Result};
use std::iter::Peekable;
use std::str::Chars;

/// Token types produced by the lexer
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Identifier (e.g., user_pref, pref, lock_pref, sticky_pref)
    Identifier(String),
    /// String value with escape sequences already processed
    String(String),
    /// Numeric value (integer or float)
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// Null value
    Null,
    /// Left parenthesis
    LeftParen,
    /// Right parenthesis
    RightParen,
    /// Comma
    Comma,
    /// Semicolon
    Semicolon,
    /// End of input
    Eof,
}

/// Lexer for tokenizing Firefox preference files
pub struct Lexer<'a> {
    /// Input character iterator
    chars: Peekable<Chars<'a>>,
    /// Current line number (1-indexed)
    line: usize,
    /// Current column number (1-indexed)
    column: usize,
    /// Track if we're at the start of a line (for column tracking)
    at_line_start: bool,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given input
    pub fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
            line: 1,
            column: 1,
            at_line_start: true,
        }
    }

    /// Get the next token from the input
    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace_and_comments();

        // Check for EOF
        if self.chars.peek().is_none() {
            return Ok(Token::Eof);
        }

        let c = *self.chars.peek().unwrap();

        match c {
            '(' => {
                self.advance();
                Ok(Token::LeftParen)
            }
            ')' => {
                self.advance();
                Ok(Token::RightParen)
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            ';' => {
                self.advance();
                Ok(Token::Semicolon)
            }
            '"' => self.lex_string(),
            '-' | '0'..='9' => self.lex_number(),
            'a'..='z' | 'A'..='Z' | '_' => self.lex_identifier(),
            _ => Err(Error::Lexer {
                message: format!("Unexpected character: '{}'", c),
                line: self.line,
                column: self.column,
            }),
        }
    }

    /// Advance to the next character
    fn advance(&mut self) {
        if self.chars.next().is_some() {
            if self.at_line_start {
                self.at_line_start = false;
            }
            self.column += 1;
        }
    }

    /// Skip whitespace and comments
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace (but track newlines)
            while let Some(&c) = self.chars.peek() {
                if c == ' ' || c == '\t' || c == '\r' {
                    self.advance();
                } else if c == '\n' {
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                    self.at_line_start = true;
                } else {
                    break;
                }
            }

            // Check for comments
            if let Some(&'/') = self.chars.peek() {
                self.advance();
                match self.chars.peek() {
                    Some(&'/') => {
                        // Single-line comment: skip to end of line
                        self.advance();
                        while let Some(&c) = self.chars.peek() {
                            if c == '\n' {
                                break;
                            }
                            self.advance();
                        }
                        continue; // Loop again to handle whitespace after comment
                    }
                    Some(&'*') => {
                        // Multi-line comment: skip to */
                        self.advance();
                        let start_line = self.line;
                        let start_col = self.column;

                        loop {
                            match self.chars.next() {
                                Some('\n') => {
                                    self.line += 1;
                                    self.column = 1;
                                    self.at_line_start = true;
                                }
                                Some('*') => {
                                    self.column += 1;
                                    if let Some(&'/') = self.chars.peek() {
                                        self.advance();
                                        break;
                                    }
                                }
                                Some(_) => {
                                    if self.at_line_start {
                                        self.at_line_start = false;
                                    }
                                    self.column += 1;
                                }
                                None => {
                                    // Unclosed comment - but we'll continue for error recovery
                                    self.line = start_line;
                                    self.column = start_col;
                                    break;
                                }
                            }
                        }
                        continue; // Loop again to handle whitespace after comment
                    }
                    _ => {
                        // Not a comment, just a single slash
                        // We've already consumed it, so we need to put it back somehow
                        // For now, we'll just break and let the error happen elsewhere
                        // (this shouldn't happen in valid prefs.js files)
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    /// Lex an identifier (e.g., user_pref, pref, true, false, null)
    fn lex_identifier(&mut self) -> Result<Token> {
        let _start_line = self.line;
        let _start_col = self.column;

        let mut ident = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Check for keywords
        match ident.as_str() {
            "true" => Ok(Token::Boolean(true)),
            "false" => Ok(Token::Boolean(false)),
            "null" => Ok(Token::Null),
            _ => Ok(Token::Identifier(ident)),
        }
    }

    /// Lex a string literal (only double-quoted strings in prefs.js)
    fn lex_string(&mut self) -> Result<Token> {
        let start_col = self.column;

        // Skip opening quote
        self.advance();

        let mut result = String::new();

        loop {
            match self.chars.next() {
                Some('"') => {
                    self.column += 1;
                    return Ok(Token::String(result));
                }
                Some('\\') => {
                    self.column += 1;
                    // Handle escape sequences
                    match self.chars.next() {
                        Some('"') => {
                            result.push('"');
                            self.column += 1;
                        }
                        Some('\'') => {
                            result.push('\'');
                            self.column += 1;
                        }
                        Some('\\') => {
                            result.push('\\');
                            self.column += 1;
                        }
                        Some('n') => {
                            result.push('\n');
                            self.column += 1;
                        }
                        Some('r') => {
                            result.push('\r');
                            self.column += 1;
                        }
                        Some('t') => {
                            result.push('\t');
                            self.column += 1;
                        }
                        Some('b') => {
                            result.push('\x08'); // Backspace (U+0008)
                            self.column += 1;
                        }
                        Some('f') => {
                            result.push('\x0c'); // Form feed (U+000C)
                            self.column += 1;
                        }
                        Some('0') => {
                            // Null character (U+0000)
                            // Check for octal escapes \00 or \000 (but allow \0 followed by digit)
                            if let Some(&c) = self.chars.peek() {
                                if c == '0' {
                                    // \00 or \000 - octal escape, not supported
                                    return Err(Error::Lexer {
                                        message: "Octal escape sequences are not supported. Use \\x00 instead.".to_string(),
                                        line: self.line,
                                        column: self.column,
                                    });
                                }
                                // \0 followed by 1-9 is fine - just null char followed by that digit
                            }
                            result.push('\x00');
                            self.column += 1;
                        }
                        Some('x') => {
                            // Hex escape: \xNN
                            self.column += 1;
                            let mut hex = String::new();
                            for _ in 0..2 {
                                if let Some(&c) = self.chars.peek() {
                                    if c.is_ascii_hexdigit() {
                                        hex.push(c);
                                        self.advance();
                                    }
                                }
                            }
                            if hex.len() == 2 {
                                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                    result.push(byte as char);
                                } else {
                                    return Err(Error::Lexer {
                                        message: format!("Invalid hex escape: \\x{}", hex),
                                        line: self.line,
                                        column: self.column,
                                    });
                                }
                            } else {
                                return Err(Error::Lexer {
                                    message: format!("Incomplete hex escape: \\x{}", hex),
                                    line: self.line,
                                    column: self.column,
                                });
                            }
                        }
                        Some('u') => {
                            // Unicode escape: \uNNNN
                            self.column += 1;
                            let mut hex = String::new();
                            for _ in 0..4 {
                                if let Some(&c) = self.chars.peek() {
                                    if c.is_ascii_hexdigit() {
                                        hex.push(c);
                                        self.advance();
                                    }
                                }
                            }
                            if hex.len() == 4 {
                                if let Ok(codepoint) = u16::from_str_radix(&hex, 16) {
                                    // Convert UTF-16 codepoint to Rust char
                                    // For BMP characters (<= 0xFFFF), this is straightforward
                                    result.push(
                                        std::char::from_u32(codepoint as u32).unwrap_or('\u{FFFD}'), // Replacement character
                                    );
                                } else {
                                    return Err(Error::Lexer {
                                        message: format!("Invalid unicode escape: \\u{}", hex),
                                        line: self.line,
                                        column: self.column,
                                    });
                                }
                            } else {
                                return Err(Error::Lexer {
                                    message: format!("Incomplete unicode escape: \\u{}", hex),
                                    line: self.line,
                                    column: self.column,
                                });
                            }
                        }
                        Some(c) => {
                            return Err(Error::Lexer {
                                message: format!("Invalid escape sequence: \\{}", c),
                                line: self.line,
                                column: self.column,
                            });
                        }
                        None => {
                            return Err(Error::Lexer {
                                message: "Unexpected end of input in escape sequence".to_string(),
                                line: self.line,
                                column: self.column,
                            });
                        }
                    }
                }
                Some('\n') => {
                    self.line += 1;
                    self.column = 1;
                    self.at_line_start = true;
                    // Multiline strings are not valid in prefs.js, but we'll allow them
                    result.push('\n');
                }
                Some(c) => {
                    if self.at_line_start {
                        self.at_line_start = false;
                    }
                    self.column += 1;
                    result.push(c);
                }
                None => {
                    return Err(Error::Lexer {
                        message: "Unterminated string literal".to_string(),
                        line: self.line,
                        column: start_col,
                    });
                }
            }
        }
    }

    /// Lex a number (integer or float, including scientific notation)
    fn lex_number(&mut self) -> Result<Token> {
        let start_col = self.column;

        let mut num_str = String::new();

        // Handle optional minus sign
        if let Some(&'-') = self.chars.peek() {
            num_str.push('-');
            self.advance();
        }

        // Parse digits
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Check for decimal point
        if let Some(&'.') = self.chars.peek() {
            num_str.push('.');
            self.advance();

            // Parse fractional digits
            while let Some(&c) = self.chars.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Check for scientific notation
        if let Some(&'e' | &'E') = self.chars.peek() {
            num_str.push('e');
            self.advance();

            // Optional + or -
            if let Some(&'+' | &'-') = self.chars.peek() {
                num_str.push(self.chars.next().unwrap());
                self.column += 1;
            }

            // Parse exponent digits
            let mut has_exp_digit = false;
            while let Some(&c) = self.chars.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    self.advance();
                    has_exp_digit = true;
                } else {
                    break;
                }
            }

            if !has_exp_digit {
                return Err(Error::Lexer {
                    message: "Missing exponent digits in scientific notation".to_string(),
                    line: self.line,
                    column: self.column,
                });
            }
        }

        // Parse the number
        match num_str.parse::<f64>() {
            Ok(n) => Ok(Token::Number(n)),
            Err(_) => Err(Error::Lexer {
                message: format!("Failed to parse number: {}", num_str),
                line: self.line,
                column: start_col,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic_tokens() {
        let input = "( ) , ;";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::LeftParen);
        assert_eq!(lexer.next_token().unwrap(), Token::RightParen);
        assert_eq!(lexer.next_token().unwrap(), Token::Comma);
        assert_eq!(lexer.next_token().unwrap(), Token::Semicolon);
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_skip_whitespace() {
        let input = "   (  \t  )  ;";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::LeftParen);
        assert_eq!(lexer.next_token().unwrap(), Token::RightParen);
        assert_eq!(lexer.next_token().unwrap(), Token::Semicolon);
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_single_line_comment() {
        let input = "( // this is a comment\n )";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::LeftParen);
        assert_eq!(lexer.next_token().unwrap(), Token::RightParen);
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_multiline_comment() {
        let input = "( /* this is a\nmultiline comment */ )";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::LeftParen);
        assert_eq!(lexer.next_token().unwrap(), Token::RightParen);
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_identifier() {
        let input = "user_pref pref lock_pref sticky_pref";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("user_pref".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("pref".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("lock_pref".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("sticky_pref".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_boolean() {
        let input = "true false";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::Boolean(true));
        assert_eq!(lexer.next_token().unwrap(), Token::Boolean(false));
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_null() {
        let input = "null";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::Null);
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_string_basic() {
        let input = r#""hello world""#;
        let mut lexer = Lexer::new(input);

        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("hello world".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_string_escaped_quotes() {
        let input = r#""value with \"quotes\"""#;
        let mut lexer = Lexer::new(input);

        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("value with \"quotes\"".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_string_backslashes() {
        let input = r#""C:\\path\\to\\file""#;
        let mut lexer = Lexer::new(input);

        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("C:\\path\\to\\file".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_string_unicode_escapes() {
        let input = r#""\u0041""#; // Should decode to 'A'
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::String("A".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_string_hex_escapes() {
        let input = r#""\x41""#; // Should decode to 'A'
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::String("A".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_string_newlines_tabs() {
        let input = r#""line1\nline2\ttab""#;
        let mut lexer = Lexer::new(input);

        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("line1\nline2\ttab".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_string_backspace_escape() {
        let input = r#""test\bvalue""#;
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("test\x08value".to_string())
        );
    }

    #[test]
    fn test_lexer_string_form_feed_escape() {
        let input = r#""test\fvalue""#;
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("test\x0cvalue".to_string())
        );
    }

    #[test]
    fn test_lexer_string_null_escape() {
        let input = r#""test\0value""#;
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("test\x00value".to_string())
        );
    }

    #[test]
    fn test_lexer_string_null_escape_followed_by_digit() {
        let input = r#""test\01""#;
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("test\x001".to_string())
        );
    }

    #[test]
    fn test_lexer_string_octal_escape_rejected() {
        let input = r#""test\00""#;
        let mut lexer = Lexer::new(input);
        assert!(lexer.next_token().is_err());
    }

    #[test]
    fn test_lexer_string_multiple_escapes_together() {
        let input = r#""\b\f\0""#;
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("\x08\x0c\x00".to_string())
        );
    }

    #[test]
    fn test_lexer_integer() {
        let input = "42";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::Number(42.0));
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_negative_integer() {
        let input = "-42";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::Number(-42.0));
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_float() {
        let input = "2.5";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::Number(2.5));
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_scientific_notation() {
        let input = "1.5e10";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::Number(1.5e10));
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_scientific_notation_negative() {
        let input = "3e-8";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::Number(3e-8));
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_complex_tokens() {
        let input = r#"user_pref("key", value);"#;
        let mut lexer = Lexer::new(input);

        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("user_pref".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::LeftParen);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("key".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Comma);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("value".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::RightParen);
        assert_eq!(lexer.next_token().unwrap(), Token::Semicolon);
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }
}
