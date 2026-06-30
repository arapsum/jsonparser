use crate::{JsonError, Result};

/// A lexical token produced from JSON source text.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Represents the left curly brace character '{'
    LeftBrace,
    /// Represents the right curly brace character '}'
    RightBrace,
    /// Represents the left square bracker character '['
    LeftBracket,
    /// Represents the right square bracker character ']'
    RightBracket,
    /// Represents the object key/value separator ':'.
    Colon,
    /// Represents the value separator ','.
    Comma,
    /// Represents a decoded JSON string value.
    String(
        /// Decoded string contents.
        String,
    ),
    /// Represents a JSON number parsed as `f64`.
    Number(
        /// Parsed numeric value.
        f64,
    ),
    /// Represents the literal `true`.
    True,
    /// Represents the literal `false`.
    False,
    /// Represents the literal `null`.
    Null,
}

/// A byte-oriented lexer over JSON source text.
#[derive(Debug, Clone)]
pub struct Lexer {
    /// Input bytes being tokenised.
    input: Vec<u8>,
    /// Current byte position in `input`.
    pos: usize,
}

impl Lexer {
    #[must_use]
    pub fn new(input: &str) -> Self {
        Self {
            input: input.as_bytes().to_vec(),
            pos: 0,
        }
    }

    #[must_use]
    pub fn current(&self) -> Option<u8> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    pub const fn advance(&mut self) {
        self.pos += 1;
    }

    #[must_use]
    pub fn peek(&self) -> Option<u8> {
        if self.pos + 1 < self.input.len() {
            Some(self.input[self.pos + 1])
        } else {
            None
        }
    }

    /// Reads a JSON string from the current position.
    ///
    /// The current byte is expected to be the opening double quote. The
    /// returned string is decoded, so supported escape sequences are converted
    /// to their character values.
    ///
    /// # Errors
    ///
    /// Returns [`JsonError::UnterminatedString`] if the closing quote is
    /// missing, [`JsonError::InvalidEscape`] for an unsupported escape,
    /// [`JsonError::UnterminatedEscape`] for a dangling backslash,
    /// [`JsonError::InvalidUnicodeEscape`] or
    /// [`JsonError::UnterminatedUnicodeEscape`] for malformed `\uXXXX`
    /// escapes, and [`JsonError::InvalidUnicodeCodepoint`] if the decoded
    /// codepoint is not a valid Unicode scalar value.
    pub fn read_string(&mut self) -> Result<String> {
        let start = self.pos;

        // consume the opening quote "
        self.advance();

        let mut result = String::new();

        loop {
            match self.current() {
                None => return Err(JsonError::UnterminatedString { position: start }),

                Some(b'"') => {
                    // consume closing quote "
                    self.advance();
                    return Ok(result);
                }

                Some(b'\\') => {
                    self.advance(); // consume the escape char \
                    match self.current() {
                        Some(b'"') => {
                            result.push('"');
                            self.advance();
                        }
                        Some(b'\\') => {
                            result.push('\\');
                            self.advance();
                        }
                        Some(b'/') => {
                            result.push('/');
                            self.advance();
                        }
                        Some(b'n') => {
                            result.push('\n');
                            self.advance();
                        }
                        Some(b't') => {
                            result.push('\t');
                            self.advance();
                        }
                        Some(b'r') => {
                            result.push('\r');
                            self.advance();
                        }
                        Some(b'b') => {
                            result.push('\x08');
                            self.advance();
                        }
                        Some(b'f') => {
                            result.push('\x0C');
                            self.advance();
                        }
                        Some(b'u') => {
                            self.advance(); // consume u
                            let codepoint = self.read_unicode_escape()?;
                            let ch = char::from_u32(codepoint).ok_or(
                                JsonError::InvalidUnicodeCodepoint {
                                    codepoint,
                                    position: self.pos,
                                },
                            )?;
                            result.push(ch);
                        }
                        Some(c) => {
                            return Err(JsonError::InvalidEscape {
                                character: c as char,
                                position: self.pos,
                            });
                        }
                        None => {
                            return Err(JsonError::UnterminatedEscape { position: self.pos });
                        }
                    }
                }
                Some(c) => {
                    result.push(c as char);
                    self.advance();
                }
            }
        }
    }

    /// Reads four hexadecimal digits from a `\uXXXX` escape.
    ///
    /// The current byte is expected to be the first hexadecimal digit after
    /// `\u`. On success, the lexer advances past all four digits and returns
    /// the decoded codepoint value.
    ///
    /// # Errors
    ///
    /// Returns [`JsonError::InvalidUnicodeEscape`] when any digit is not
    /// hexadecimal, or [`JsonError::UnterminatedUnicodeEscape`] if fewer than
    /// four digits remain.
    pub fn read_unicode_escape(&mut self) -> Result<u32> {
        let mut value: u32 = 0;
        for _ in 0..4 {
            match self.current() {
                Some(c) => {
                    let digit = match c {
                        b'0'..=b'9' => u32::from(c - b'0'),
                        b'a'..=b'f' => u32::from(c - b'a' + 10),
                        b'A'..=b'F' => u32::from(c - b'A' + 10),
                        _ => {
                            return Err(JsonError::InvalidUnicodeEscape {
                                character: c as char,
                                position: self.pos,
                            });
                        }
                    };
                    value = value * 16 + digit;
                    self.advance();
                }
                None => {
                    return Err(JsonError::UnterminatedUnicodeEscape { position: self.pos });
                }
            }
        }
        Ok(value)
    }

    /// Reads an exact JSON keyword from the current position.
    ///
    /// `s` is expected to be one of the JSON literals `true`, `false`, or
    /// `null`.
    ///
    /// # Errors
    ///
    /// Returns [`JsonError::UnexpectedCharacter`] when the next input byte does
    /// not match the keyword, or [`JsonError::UnexpectedEnd`] if the input ends
    /// before the keyword is complete.
    pub fn read_keyword(&mut self, s: &'static str) -> Result<()> {
        for expected in s.as_bytes() {
            match self.current() {
                Some(c) if c == *expected => self.advance(),
                Some(c) => {
                    return Err(JsonError::UnexpectedCharacter {
                        character: c as char,
                        position: self.pos,
                    });
                }
                None => {
                    return Err(JsonError::UnexpectedEnd {
                        expected: s,
                        position: self.pos,
                    });
                }
            }
        }
        Ok(())
    }

    /// Reads a JSON number from the current position.
    ///
    /// This parser supports an optional leading minus sign, digits, and an
    /// optional fractional part, then parses the collected literal as `f64`.
    ///
    /// # Errors
    ///
    /// Returns [`JsonError::InvalidNumber`] if the collected literal cannot be
    /// parsed as `f64`.
    pub fn read_number(&mut self) -> Result<f64> {
        let start = self.pos;
        let mut s = String::new();

        // optional minus sign
        if self.current() == Some(b'-') {
            s.push('-');
            self.advance();
        }

        // integer part
        while let Some(c @ b'0'..=b'9') = self.current() {
            s.push(c as char);
            self.advance();
        }

        // optional decimal part
        if self.current() == Some(b'.') {
            s.push('.');
            self.advance();

            while let Some(c @ b'0'..=b'9') = self.current() {
                s.push(c as char);
                self.advance();
            }
        }

        s.parse::<f64>().map_err(|_| JsonError::InvalidNumber {
            value: s,
            position: start,
        })
    }
}

/// Converts JSON source text into a vector of tokens.
///
/// Whitespace outside strings is skipped. String escapes are decoded while
/// tokenising.
///
/// # Errors
///
/// Returns lexer errors for malformed strings, malformed unicode escapes,
/// invalid numbers, truncated keywords, or unexpected characters.
pub fn tokenise(input: &str) -> Result<Vec<Token>> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    loop {
        match lexer.current() {
            None => break,

            Some(b' ' | b'\t' | b'\r' | b'\n') => {
                lexer.advance();
            }

            Some(b'{') => {
                tokens.push(Token::LeftBrace);
                lexer.advance();
            }
            Some(b'}') => {
                tokens.push(Token::RightBrace);
                lexer.advance();
            }
            Some(b'[') => {
                tokens.push(Token::LeftBracket);
                lexer.advance();
            }
            Some(b']') => {
                tokens.push(Token::RightBracket);
                lexer.advance();
            }
            Some(b':') => {
                tokens.push(Token::Colon);
                lexer.advance();
            }
            Some(b',') => {
                tokens.push(Token::Comma);
                lexer.advance();
            }
            Some(b'"') => {
                let s = lexer.read_string()?;
                tokens.push(Token::String(s));
            }
            Some(b't') => {
                lexer.read_keyword("true")?;
                tokens.push(Token::True);
            }
            Some(b'f') => {
                lexer.read_keyword("false")?;
                tokens.push(Token::False);
            }
            Some(b'n') => {
                lexer.read_keyword("null")?;
                tokens.push(Token::Null);
            }
            Some(b'-' | b'0'..=b'9') => {
                let n = lexer.read_number()?;
                tokens.push(Token::Number(n));
            }

            Some(c) => {
                return Err(JsonError::UnexpectedCharacter {
                    character: c as char,
                    position: lexer.pos,
                });
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Lexer basics ──────────────────────────────────────────────

    #[test]
    fn test_lexer_new_and_current() {
        let lexer = Lexer::new("abc");
        assert_eq!(lexer.current(), Some(b'a'));
    }

    #[test]
    fn test_lexer_current_at_end() {
        let lexer = Lexer::new("");
        assert_eq!(lexer.current(), None);
    }

    #[test]
    fn test_lexer_advance() {
        let mut lexer = Lexer::new("ab");
        assert_eq!(lexer.current(), Some(b'a'));
        lexer.advance();
        assert_eq!(lexer.current(), Some(b'b'));
        lexer.advance();
        assert_eq!(lexer.current(), None);
    }

    #[test]
    fn test_lexer_peek() {
        let mut lexer = Lexer::new("ab");
        assert_eq!(lexer.peek(), Some(b'b'));
        lexer.advance();
        assert_eq!(lexer.peek(), None);
    }

    #[test]
    fn test_lexer_peek_at_end() {
        let lexer = Lexer::new("a");
        assert_eq!(lexer.peek(), None);
    }

    #[test]
    fn test_lexer_peek_empty() {
        let lexer = Lexer::new("");
        assert_eq!(lexer.peek(), None);
    }

    // ── read_string – basic & escapes ─────────────────────────────

    #[test]
    fn test_read_string_basic() {
        let mut lexer = Lexer::new(r#""hello""#);
        assert_eq!(lexer.read_string().unwrap(), "hello");
        assert_eq!(lexer.current(), None);
    }

    #[test]
    fn test_read_string_empty() {
        let mut lexer = Lexer::new(r#""""#);
        assert_eq!(lexer.read_string().unwrap(), "");
    }

    #[test]
    fn test_read_string_with_spaces() {
        let mut lexer = Lexer::new(r#""hello world""#);
        assert_eq!(lexer.read_string().unwrap(), "hello world");
    }

    #[test]
    fn test_read_string_escape_quote() {
        let mut lexer = Lexer::new(r#""\"""#);
        assert_eq!(lexer.read_string().unwrap(), "\"");
    }

    #[test]
    fn test_read_string_escape_backslash() {
        let mut lexer = Lexer::new(r#""\\""#);
        assert_eq!(lexer.read_string().unwrap(), "\\");
    }

    #[test]
    fn test_read_string_escape_slash() {
        let mut lexer = Lexer::new(r#""\/""#);
        assert_eq!(lexer.read_string().unwrap(), "/");
    }

    #[test]
    fn test_read_string_escape_newline() {
        let mut lexer = Lexer::new(r#""\n""#);
        assert_eq!(lexer.read_string().unwrap(), "\n");
    }

    #[test]
    fn test_read_string_escape_tab() {
        let mut lexer = Lexer::new(r#""\t""#);
        assert_eq!(lexer.read_string().unwrap(), "\t");
    }

    #[test]
    fn test_read_string_escape_return() {
        let mut lexer = Lexer::new(r#""\r""#);
        assert_eq!(lexer.read_string().unwrap(), "\r");
    }

    #[test]
    fn test_read_string_escape_backspace() {
        let mut lexer = Lexer::new(r#""\b""#);
        assert_eq!(lexer.read_string().unwrap(), "\x08");
    }

    #[test]
    fn test_read_string_escape_formfeed() {
        let mut lexer = Lexer::new(r#""\f""#);
        assert_eq!(lexer.read_string().unwrap(), "\x0C");
    }

    #[test]
    fn test_read_string_all_escapes() {
        let input = concat!(
            "\"",   // opening "
            "\\\"", // \"
            "\\\\", // \\
            "\\/",  // \/
            "\\n",  // \n
            "\\t",  // \t
            "\\r",  // \r
            "\\b",  // \b
            "\\f",  // \f
            "\"",   // closing "
        );
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.read_string().unwrap(), "\"\\/\n\t\r\x08\x0C");
    }

    #[test]
    fn test_read_string_unicode_escape() {
        let mut lexer = Lexer::new(r#""\u0041""#);
        assert_eq!(lexer.read_string().unwrap(), "A");
    }

    #[test]
    fn test_read_string_unicode_escape_lowercase_hex() {
        let mut lexer = Lexer::new(r#""\u00ff""#);
        assert_eq!(lexer.read_string().unwrap(), "\u{ff}");
    }

    #[test]
    fn test_read_string_unicode_escape_mixed_case() {
        let mut lexer = Lexer::new(r#""\u00Ff""#);
        assert_eq!(lexer.read_string().unwrap(), "\u{ff}");
    }

    #[test]
    fn test_read_string_unicode_snowman() {
        let mut lexer = Lexer::new(r#""\u2603""#);
        assert_eq!(lexer.read_string().unwrap(), "☃");
    }

    #[test]
    fn test_read_string_multiple_unicode() {
        let mut lexer = Lexer::new(r#""\u0048\u0065\u006c\u006c\u006f""#);
        assert_eq!(lexer.read_string().unwrap(), "Hello");
    }

    #[test]
    fn test_read_string_mixed_escapes_and_text() {
        let input = concat!(
            "\"",       // opening "
            "line1",    //
            "\\n",      // \n
            "line2",    //
            "\\t",      // \t
            "indented", //
            "\\\"",     // \"
            "\"",       // closing "
        );
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.read_string().unwrap(), "line1\nline2\tindented\"");
    }

    // ── read_string – error cases ─────────────────────────────────

    #[test]
    #[should_panic]
    fn test_read_string_unterminated() {
        let mut lexer = Lexer::new(r#""hello"#);
        lexer.read_string().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_read_string_invalid_escape() {
        let mut lexer = Lexer::new(r#""\x""#);
        lexer.read_string().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_read_string_invalid_escape_letter() {
        let mut lexer = Lexer::new(r#""\a""#);
        lexer.read_string().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_read_string_unterminated_escape() {
        let mut lexer = Lexer::new(r#""\"#);
        lexer.read_string().unwrap();
    }

    // ── read_unicode_escape ───────────────────────────────────────

    #[test]
    fn test_read_unicode_escape_basic() {
        let mut lexer = Lexer::new("0041");
        assert_eq!(lexer.read_unicode_escape().unwrap(), 0x0041);
    }

    #[test]
    #[should_panic]
    fn test_read_unicode_escape_invalid_hex() {
        let mut lexer = Lexer::new("00GG");
        lexer.read_unicode_escape().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_read_unicode_escape_unterminated() {
        let mut lexer = Lexer::new("00");
        lexer.read_unicode_escape().unwrap();
    }

    // ── read_number ───────────────────────────────────────────────

    #[test]
    fn test_read_number_integer() {
        let mut lexer = Lexer::new("42");
        assert_eq!(lexer.read_number().unwrap(), 42.0);
    }

    #[test]
    fn test_read_number_negative() {
        let mut lexer = Lexer::new("-42");
        assert_eq!(lexer.read_number().unwrap(), -42.0);
    }

    #[test]
    fn test_read_number_decimal() {
        let mut lexer = Lexer::new("3.19");
        assert_eq!(lexer.read_number().unwrap(), 3.19);
    }

    #[test]
    fn test_read_number_negative_decimal() {
        let mut lexer = Lexer::new("-3.19");
        assert_eq!(lexer.read_number().unwrap(), -3.19);
    }

    #[test]
    fn test_read_number_zero() {
        let mut lexer = Lexer::new("0");
        assert_eq!(lexer.read_number().unwrap(), 0.0);
    }

    #[test]
    fn test_read_number_negative_zero() {
        let mut lexer = Lexer::new("-0");
        assert_eq!(lexer.read_number().unwrap(), -0.0);
    }

    #[test]
    fn test_read_number_zero_decimal() {
        let mut lexer = Lexer::new("0.5");
        assert_eq!(lexer.read_number().unwrap(), 0.5);
    }

    #[test]
    fn test_read_number_large() {
        let mut lexer = Lexer::new("999999");
        assert_eq!(lexer.read_number().unwrap(), 999999.0);
    }

    #[test]
    fn test_read_number_stops_at_non_digit() {
        let mut lexer = Lexer::new("42 }");
        assert_eq!(lexer.read_number().unwrap(), 42.0);
        assert_eq!(lexer.current(), Some(b' '));
    }

    #[test]
    fn test_read_negative_number_stops_at_non_digit() {
        let mut lexer = Lexer::new("-42,");
        assert_eq!(lexer.read_number().unwrap(), -42.0);
        assert_eq!(lexer.current(), Some(b','));
    }

    // ── read_keyword ──────────────────────────────────────────────

    #[test]
    fn test_read_keyword_true() {
        let mut lexer = Lexer::new("true");
        lexer.read_keyword("true").unwrap();
        assert_eq!(lexer.current(), None);
    }

    #[test]
    fn test_read_keyword_false() {
        let mut lexer = Lexer::new("false");
        lexer.read_keyword("false").unwrap();
        assert_eq!(lexer.current(), None);
    }

    #[test]
    fn test_read_keyword_null() {
        let mut lexer = Lexer::new("null");
        lexer.read_keyword("null").unwrap();
        assert_eq!(lexer.current(), None);
    }

    #[test]
    #[should_panic]
    fn test_read_keyword_invalid() {
        let mut lexer = Lexer::new("trux");
        lexer.read_keyword("true").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_read_keyword_truncated() {
        let mut lexer = Lexer::new("tr");
        lexer.read_keyword("true").unwrap();
    }

    // ── tokenise – empty / whitespace ─────────────────────────────

    #[test]
    fn test_tokenise_empty_input() {
        let tokens = tokenise("").unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenise_whitespace_only() {
        let tokens = tokenise("   \t\n\r   ").unwrap();
        assert!(tokens.is_empty());
    }

    // ── tokenise – single tokens ──────────────────────────────────

    #[test]
    fn test_tokenise_left_brace() {
        assert_eq!(tokenise("{").unwrap(), vec![Token::LeftBrace]);
    }

    #[test]
    fn test_tokenise_right_brace() {
        assert_eq!(tokenise("}").unwrap(), vec![Token::RightBrace]);
    }

    #[test]
    fn test_tokenise_left_bracket() {
        assert_eq!(tokenise("[").unwrap(), vec![Token::LeftBracket]);
    }

    #[test]
    fn test_tokenise_right_bracket() {
        assert_eq!(tokenise("]").unwrap(), vec![Token::RightBracket]);
    }

    #[test]
    fn test_tokenise_colon() {
        assert_eq!(tokenise(":").unwrap(), vec![Token::Colon]);
    }

    #[test]
    fn test_tokenise_comma() {
        assert_eq!(tokenise(",").unwrap(), vec![Token::Comma]);
    }

    #[test]
    fn test_tokenise_true() {
        assert_eq!(tokenise("true").unwrap(), vec![Token::True]);
    }

    #[test]
    fn test_tokenise_false() {
        assert_eq!(tokenise("false").unwrap(), vec![Token::False]);
    }

    #[test]
    fn test_tokenise_null() {
        assert_eq!(tokenise("null").unwrap(), vec![Token::Null]);
    }

    #[test]
    fn test_tokenise_string() {
        assert_eq!(
            tokenise(r#""hello""#).unwrap(),
            vec![Token::String("hello".to_string())]
        );
    }

    #[test]
    fn test_tokenise_number_integer() {
        assert_eq!(tokenise("42").unwrap(), vec![Token::Number(42.0)]);
    }

    #[test]
    fn test_tokenise_number_negative() {
        assert_eq!(tokenise("-42").unwrap(), vec![Token::Number(-42.0)]);
    }

    #[test]
    fn test_tokenise_number_decimal() {
        assert_eq!(tokenise("3.19").unwrap(), vec![Token::Number(3.19)]);
    }

    // ── tokenise – multiple tokens ────────────────────────────────

    #[test]
    fn test_tokenise_symbols() {
        assert_eq!(
            tokenise("{}[]:,").unwrap(),
            vec![
                Token::LeftBrace,
                Token::RightBrace,
                Token::LeftBracket,
                Token::RightBracket,
                Token::Colon,
                Token::Comma,
            ]
        );
    }

    #[test]
    fn test_tokenise_mixed_whitespace() {
        let tokens = tokenise("  \t\r\n{\n\t}  ").unwrap();
        assert_eq!(tokens, vec![Token::LeftBrace, Token::RightBrace]);
    }

    #[test]
    fn test_tokenise_multiple_strings() {
        assert_eq!(
            tokenise(r#""a" "b" "c""#).unwrap(),
            vec![
                Token::String("a".to_string()),
                Token::String("b".to_string()),
                Token::String("c".to_string()),
            ]
        );
    }

    #[test]
    fn test_tokenise_multiple_numbers() {
        assert_eq!(
            tokenise("1 2 3").unwrap(),
            vec![Token::Number(1.0), Token::Number(2.0), Token::Number(3.0)]
        );
    }

    #[test]
    fn test_tokenise_simple_object() {
        assert_eq!(
            tokenise(r#"{"k":"v"}"#).unwrap(),
            vec![
                Token::LeftBrace,
                Token::String("k".to_string()),
                Token::Colon,
                Token::String("v".to_string()),
                Token::RightBrace,
            ]
        );
    }

    #[test]
    fn test_tokenise_simple_array() {
        assert_eq!(
            tokenise(r#"[1,2,3]"#).unwrap(),
            vec![
                Token::LeftBracket,
                Token::Number(1.0),
                Token::Comma,
                Token::Number(2.0),
                Token::Comma,
                Token::Number(3.0),
                Token::RightBracket,
            ]
        );
    }

    #[test]
    fn test_tokenise_keyword_after_whitespace() {
        assert_eq!(tokenise("  true  ").unwrap(), vec![Token::True]);
        assert_eq!(tokenise("\tfalse\n").unwrap(), vec![Token::False]);
        assert_eq!(tokenise("\r\nnull\r\n").unwrap(), vec![Token::Null]);
    }

    // ── tokenise – error cases ────────────────────────────────────

    #[test]
    #[should_panic]
    fn test_tokenise_unexpected_at_symbol() {
        tokenise("@").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_tokenise_unexpected_tilde() {
        tokenise("~").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_tokenise_unexpected_control_char() {
        tokenise("\x01").unwrap();
    }
}
