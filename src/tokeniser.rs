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
    Colon,
    Comma,
    String(String),
    Number(f64),
    True,
    False,
    Null,
}

#[derive(Debug, Clone)]
pub struct Lexer {
    input: Vec<u8>,
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

    pub fn advance(&mut self) {
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

    pub fn read_string(&mut self) -> String {
        // consume the opening quote "
        self.advance();

        let mut result = String::new();

        loop {
            match self.current() {
                None => panic!("Unterminated String"),

                Some(b'"') => {
                    // consume closing quote "
                    self.advance();
                    return result;
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
                            let codepoint = self.read_unicode_escape();
                            let ch = char::from_u32(codepoint).unwrap_or_else(|| {
                                panic!("Invalid unicode codepoint: {codepoint}")
                            });
                            result.push(ch);
                        }
                        Some(c) => panic!("Inalid escape sequence: \\{}", c as char),
                        None => panic!("Unterminated escape sequence"),
                    }
                }
                Some(c) => {
                    result.push(c as char);
                    self.advance();
                }
            }
        }
    }

    pub fn read_unicode_escape(&mut self) -> u32 {
        let mut value: u32 = 0;
        for _ in 0..4 {
            match self.current() {
                Some(c) => {
                    let digit = match c {
                        b'0'..=b'9' => u32::from(c - b'0'),
                        b'a'..=b'f' => u32::from(c - b'a' + 10),
                        b'A'..=b'F' => u32::from(c - b'A' + 10),
                        _ => panic!("Invalid hex digit in unicode escape: {}", c as char),
                    };
                    value = value * 16 + digit;
                    self.advance();
                }
                None => panic!("Unterminated unicode escape"),
            }
        }
        value
    }

    pub fn read_keyword(&mut self, s: &str) {
        for expected in s.as_bytes() {
            match self.current() {
                Some(c) if c == *expected => self.advance(),
                Some(c) => panic!("Unexpected character '{c}' while reading keyword '{s}'"),
                None => panic!("Unexpected end of input while reading keyword '{s}'"),
            }
        }
    }

    pub fn read_number(&mut self) -> f64 {
        let mut s = String::new();

        // optional minus sign
        if let Some(b'-') = self.current() {
            s.push('-');
            self.advance();
        }

        // integer part
        while let Some(c @ b'0'..=b'9') = self.current() {
            s.push(c as char);
            self.advance();
        }

        // optional decimal part
        if let Some(b'.') = self.current() {
            s.push('.');
            self.advance();

            while let Some(c @ b'0'..=b'9') = self.current() {
                s.push(c as char);
                self.advance();
            }
        }

        s.parse::<f64>()
            .unwrap_or_else(|_| panic!("Invalid number: {s}"))
    }
}

#[must_use] 
pub fn tokenise(input: &str) -> Vec<Token> {
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
                let s = lexer.read_string();
                tokens.push(Token::String(s));
            }
            Some(b't') => {
                lexer.read_keyword("true");
                tokens.push(Token::True);
            }
            Some(b'f') => {
                lexer.read_keyword("false");
                tokens.push(Token::False);
            }
            Some(b'n') => {
                lexer.read_keyword("null");
                tokens.push(Token::Null);
            }
            Some(b'-' | b'0'..=b'9') => {
                let n = lexer.read_number();
                tokens.push(Token::Number(n));
            }

            Some(c) => {
                panic!("Unexpected character {}", c as char);
            }
        }
    }

    tokens
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
        assert_eq!(lexer.read_string(), "hello");
        assert_eq!(lexer.current(), None);
    }

    #[test]
    fn test_read_string_empty() {
        let mut lexer = Lexer::new(r#""""#);
        assert_eq!(lexer.read_string(), "");
    }

    #[test]
    fn test_read_string_with_spaces() {
        let mut lexer = Lexer::new(r#""hello world""#);
        assert_eq!(lexer.read_string(), "hello world");
    }

    #[test]
    fn test_read_string_escape_quote() {
        let mut lexer = Lexer::new(r#""\"""#);
        assert_eq!(lexer.read_string(), "\"");
    }

    #[test]
    fn test_read_string_escape_backslash() {
        let mut lexer = Lexer::new(r#""\\""#);
        assert_eq!(lexer.read_string(), "\\");
    }

    #[test]
    fn test_read_string_escape_slash() {
        let mut lexer = Lexer::new(r#""\/""#);
        assert_eq!(lexer.read_string(), "/");
    }

    #[test]
    fn test_read_string_escape_newline() {
        let mut lexer = Lexer::new(r#""\n""#);
        assert_eq!(lexer.read_string(), "\n");
    }

    #[test]
    fn test_read_string_escape_tab() {
        let mut lexer = Lexer::new(r#""\t""#);
        assert_eq!(lexer.read_string(), "\t");
    }

    #[test]
    fn test_read_string_escape_return() {
        let mut lexer = Lexer::new(r#""\r""#);
        assert_eq!(lexer.read_string(), "\r");
    }

    #[test]
    fn test_read_string_escape_backspace() {
        let mut lexer = Lexer::new(r#""\b""#);
        assert_eq!(lexer.read_string(), "\x08");
    }

    #[test]
    fn test_read_string_escape_formfeed() {
        let mut lexer = Lexer::new(r#""\f""#);
        assert_eq!(lexer.read_string(), "\x0C");
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
        assert_eq!(lexer.read_string(), "\"\\/\n\t\r\x08\x0C");
    }

    #[test]
    fn test_read_string_unicode_escape() {
        let mut lexer = Lexer::new(r#""\u0041""#);
        assert_eq!(lexer.read_string(), "A");
    }

    #[test]
    fn test_read_string_unicode_escape_lowercase_hex() {
        let mut lexer = Lexer::new(r#""\u00ff""#);
        assert_eq!(lexer.read_string(), "\u{ff}");
    }

    #[test]
    fn test_read_string_unicode_escape_mixed_case() {
        let mut lexer = Lexer::new(r#""\u00Ff""#);
        assert_eq!(lexer.read_string(), "\u{ff}");
    }

    #[test]
    fn test_read_string_unicode_snowman() {
        let mut lexer = Lexer::new(r#""\u2603""#);
        assert_eq!(lexer.read_string(), "☃");
    }

    #[test]
    fn test_read_string_multiple_unicode() {
        let mut lexer = Lexer::new(r#""\u0048\u0065\u006c\u006c\u006f""#);
        assert_eq!(lexer.read_string(), "Hello");
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
        assert_eq!(lexer.read_string(), "line1\nline2\tindented\"");
    }

    // ── read_string – error cases ─────────────────────────────────

    #[test]
    #[should_panic(expected = "Unterminated String")]
    fn test_read_string_unterminated() {
        let mut lexer = Lexer::new(r#""hello"#);
        lexer.read_string();
    }

    #[test]
    #[should_panic(expected = "Inalid escape sequence")]
    fn test_read_string_invalid_escape() {
        let mut lexer = Lexer::new(r#""\x""#);
        lexer.read_string();
    }

    #[test]
    #[should_panic(expected = "Inalid escape sequence")]
    fn test_read_string_invalid_escape_letter() {
        let mut lexer = Lexer::new(r#""\a""#);
        lexer.read_string();
    }

    #[test]
    #[should_panic(expected = "Unterminated escape sequence")]
    fn test_read_string_unterminated_escape() {
        let mut lexer = Lexer::new(r#""\"#);
        lexer.read_string();
    }

    // ── read_unicode_escape ───────────────────────────────────────

    #[test]
    fn test_read_unicode_escape_basic() {
        let mut lexer = Lexer::new("0041");
        assert_eq!(lexer.read_unicode_escape(), 0x0041);
    }

    #[test]
    #[should_panic(expected = "Invalid hex digit")]
    fn test_read_unicode_escape_invalid_hex() {
        let mut lexer = Lexer::new("00GG");
        lexer.read_unicode_escape();
    }

    #[test]
    #[should_panic(expected = "Unterminated unicode escape")]
    fn test_read_unicode_escape_unterminated() {
        let mut lexer = Lexer::new("00");
        lexer.read_unicode_escape();
    }

    // ── read_number ───────────────────────────────────────────────

    #[test]
    fn test_read_number_integer() {
        let mut lexer = Lexer::new("42");
        assert_eq!(lexer.read_number(), 42.0);
    }

    #[test]
    fn test_read_number_negative() {
        let mut lexer = Lexer::new("-42");
        assert_eq!(lexer.read_number(), -42.0);
    }

    #[test]
    fn test_read_number_decimal() {
        let mut lexer = Lexer::new("3.14");
        assert_eq!(lexer.read_number(), 3.14);
    }

    #[test]
    fn test_read_number_negative_decimal() {
        let mut lexer = Lexer::new("-3.14");
        assert_eq!(lexer.read_number(), -3.14);
    }

    #[test]
    fn test_read_number_zero() {
        let mut lexer = Lexer::new("0");
        assert_eq!(lexer.read_number(), 0.0);
    }

    #[test]
    fn test_read_number_negative_zero() {
        let mut lexer = Lexer::new("-0");
        assert_eq!(lexer.read_number(), -0.0);
    }

    #[test]
    fn test_read_number_zero_decimal() {
        let mut lexer = Lexer::new("0.5");
        assert_eq!(lexer.read_number(), 0.5);
    }

    #[test]
    fn test_read_number_large() {
        let mut lexer = Lexer::new("999999");
        assert_eq!(lexer.read_number(), 999999.0);
    }

    #[test]
    fn test_read_number_stops_at_non_digit() {
        let mut lexer = Lexer::new("42 }");
        assert_eq!(lexer.read_number(), 42.0);
        assert_eq!(lexer.current(), Some(b' '));
    }

    #[test]
    fn test_read_negative_number_stops_at_non_digit() {
        let mut lexer = Lexer::new("-42,");
        assert_eq!(lexer.read_number(), -42.0);
        assert_eq!(lexer.current(), Some(b','));
    }

    // ── read_keyword ──────────────────────────────────────────────

    #[test]
    fn test_read_keyword_true() {
        let mut lexer = Lexer::new("true");
        lexer.read_keyword("true");
        assert_eq!(lexer.current(), None);
    }

    #[test]
    fn test_read_keyword_false() {
        let mut lexer = Lexer::new("false");
        lexer.read_keyword("false");
        assert_eq!(lexer.current(), None);
    }

    #[test]
    fn test_read_keyword_null() {
        let mut lexer = Lexer::new("null");
        lexer.read_keyword("null");
        assert_eq!(lexer.current(), None);
    }

    #[test]
    #[should_panic(expected = "Unexpected character")]
    fn test_read_keyword_invalid() {
        let mut lexer = Lexer::new("trux");
        lexer.read_keyword("true");
    }

    #[test]
    #[should_panic(expected = "Unexpected end of input")]
    fn test_read_keyword_truncated() {
        let mut lexer = Lexer::new("tr");
        lexer.read_keyword("true");
    }

    // ── tokenise – empty / whitespace ─────────────────────────────

    #[test]
    fn test_tokenise_empty_input() {
        let tokens = tokenise("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenise_whitespace_only() {
        let tokens = tokenise("   \t\n\r   ");
        assert!(tokens.is_empty());
    }

    // ── tokenise – single tokens ──────────────────────────────────

    #[test]
    fn test_tokenise_left_brace() {
        assert_eq!(tokenise("{"), vec![Token::LeftBrace]);
    }

    #[test]
    fn test_tokenise_right_brace() {
        assert_eq!(tokenise("}"), vec![Token::RightBrace]);
    }

    #[test]
    fn test_tokenise_left_bracket() {
        assert_eq!(tokenise("["), vec![Token::LeftBracket]);
    }

    #[test]
    fn test_tokenise_right_bracket() {
        assert_eq!(tokenise("]"), vec![Token::RightBracket]);
    }

    #[test]
    fn test_tokenise_colon() {
        assert_eq!(tokenise(":"), vec![Token::Colon]);
    }

    #[test]
    fn test_tokenise_comma() {
        assert_eq!(tokenise(","), vec![Token::Comma]);
    }

    #[test]
    fn test_tokenise_true() {
        assert_eq!(tokenise("true"), vec![Token::True]);
    }

    #[test]
    fn test_tokenise_false() {
        assert_eq!(tokenise("false"), vec![Token::False]);
    }

    #[test]
    fn test_tokenise_null() {
        assert_eq!(tokenise("null"), vec![Token::Null]);
    }

    #[test]
    fn test_tokenise_string() {
        assert_eq!(
            tokenise(r#""hello""#),
            vec![Token::String("hello".to_string())]
        );
    }

    #[test]
    fn test_tokenise_number_integer() {
        assert_eq!(tokenise("42"), vec![Token::Number(42.0)]);
    }

    #[test]
    fn test_tokenise_number_negative() {
        assert_eq!(tokenise("-42"), vec![Token::Number(-42.0)]);
    }

    #[test]
    fn test_tokenise_number_decimal() {
        assert_eq!(tokenise("3.14"), vec![Token::Number(3.14)]);
    }

    // ── tokenise – multiple tokens ────────────────────────────────

    #[test]
    fn test_tokenise_symbols() {
        assert_eq!(
            tokenise("{}[]:,"),
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
        let tokens = tokenise("  \t\r\n{\n\t}  ");
        assert_eq!(tokens, vec![Token::LeftBrace, Token::RightBrace]);
    }

    #[test]
    fn test_tokenise_multiple_strings() {
        assert_eq!(
            tokenise(r#""a" "b" "c""#),
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
            tokenise("1 2 3"),
            vec![Token::Number(1.0), Token::Number(2.0), Token::Number(3.0)]
        );
    }

    #[test]
    fn test_tokenise_simple_object() {
        assert_eq!(
            tokenise(r#"{"k":"v"}"#),
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
            tokenise(r#"[1,2,3]"#),
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
        assert_eq!(tokenise("  true  "), vec![Token::True]);
        assert_eq!(tokenise("\tfalse\n"), vec![Token::False]);
        assert_eq!(tokenise("\r\nnull\r\n"), vec![Token::Null]);
    }

    // ── tokenise – error cases ────────────────────────────────────

    #[test]
    #[should_panic(expected = "Unexpected character")]
    fn test_tokenise_unexpected_at_symbol() {
        tokenise("@");
    }

    #[test]
    #[should_panic(expected = "Unexpected character")]
    fn test_tokenise_unexpected_tilde() {
        tokenise("~");
    }

    #[test]
    #[should_panic(expected = "Unexpected character")]
    fn test_tokenise_unexpected_control_char() {
        tokenise("\x01");
    }
}
