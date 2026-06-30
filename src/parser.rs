use crate::{JsonError, JsonValue, Result, Token};

/// Recursive-descent parser over a token stream.
#[derive(Debug, Clone)]
pub struct Parser {
    /// Tokens being parsed.
    tokens: Vec<Token>,
    /// Current token position in `tokens`.
    pos: usize,
}

impl Parser {
    #[must_use]
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Returns the current token without advancing.
    ///
    /// # Errors
    ///
    /// Returns [`JsonError::UnexpectedEnd`] if the parser is already past the
    /// end of the token stream.
    pub fn current(&self) -> Result<&Token> {
        if self.pos < self.tokens.len() {
            Ok(&self.tokens[self.pos])
        } else {
            Err(JsonError::UnexpectedEnd {
                expected: "JSON value",
                position: self.pos,
            })
        }
    }

    pub fn advance(&mut self) {
        self.pos += 1;
    }

    /// Consumes and returns the current token.
    ///
    /// `expected` is only used to describe the error if the stream has already
    /// ended.
    ///
    /// # Errors
    ///
    /// Returns [`JsonError::UnexpectedEnd`] if there is no current token to
    /// consume.
    pub fn expect(&mut self, expected: &'static str) -> Result<Token> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Ok(token)
        } else {
            Err(JsonError::UnexpectedEnd {
                expected,
                position: self.pos,
            })
        }
    }

    /// Parses a JSON value from the current token.
    ///
    /// Values may be objects, arrays, strings, numbers, booleans, or `null`.
    ///
    /// # Errors
    ///
    /// Returns [`JsonError::UnexpectedEnd`] if no value is available, or
    /// [`JsonError::UnexpectedToken`] if the current token cannot begin a JSON
    /// value. Errors from nested object and array parsing are propagated.
    pub fn parse_value(&mut self) -> Result<JsonValue> {
        let position = self.pos;

        match self.current()?.clone() {
            Token::LeftBrace => self.parse_object(),
            Token::LeftBracket => self.parse_array(),
            Token::True => {
                self.advance();
                Ok(JsonValue::Boolean(true))
            }
            Token::False => {
                self.advance();
                Ok(JsonValue::Boolean(false))
            }
            Token::Null => {
                self.advance();
                Ok(JsonValue::Null)
            }
            Token::Number(n) => {
                self.advance();
                Ok(JsonValue::Number(n))
            }
            Token::String(s) => {
                self.advance();
                Ok(JsonValue::String(s))
            }
            found @ (Token::RightBrace | Token::RightBracket | Token::Colon | Token::Comma) => {
                Err(JsonError::UnexpectedToken {
                    expected: "JSON value",
                    found,
                    position,
                })
            }
        }
    }

    /// Parses a JSON array from the current token.
    ///
    /// The current token is expected to be [`Token::LeftBracket`]. On success,
    /// the parser advances past the closing bracket and returns
    /// [`JsonValue::Array`].
    ///
    /// # Errors
    ///
    /// Returns [`JsonError::UnexpectedEnd`] if the array is not closed, or
    /// [`JsonError::UnexpectedToken`] when an element, comma, or closing bracket
    /// is missing or malformed. Errors from nested values are propagated.
    pub fn parse_array(&mut self) -> Result<JsonValue> {
        self.advance(); // consume the opening paranthesis

        let mut array = Vec::new();
        // empty array
        if let Token::RightBracket = self.current()? {
            self.advance();
            return Ok(JsonValue::Array(array));
        }

        loop {
            let element = self.parse_value()?; // recursive call
            array.push(element);

            match self.current()? {
                Token::Comma => {
                    self.advance(); // consume comma, seek next el
                }
                Token::RightBracket => {
                    self.advance(); // consume closing paranthesis, we are at the end
                    break;
                }
                found => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "',' or ']'",
                        found: found.clone(),
                        position: self.pos,
                    });
                }
            }
        }

        Ok(JsonValue::Array(array))
    }

    /// Parses a JSON object from the current token.
    ///
    /// The current token is expected to be [`Token::LeftBrace`]. On success,
    /// the parser advances past the closing brace and returns
    /// [`JsonValue::Object`].
    ///
    /// # Errors
    ///
    /// Returns [`JsonError::UnexpectedEnd`] if the object is not closed, or
    /// [`JsonError::UnexpectedToken`] when a string key, colon, comma, closing
    /// brace, or nested value is missing or malformed.
    pub fn parse_object(&mut self) -> Result<JsonValue> {
        self.advance();

        let mut pairs: Vec<(String, JsonValue)> = Vec::new();

        // object may be empty
        if let Token::RightBrace = self.current()? {
            self.advance();
            return Ok(JsonValue::Object(pairs));
        }

        loop {
            // parse the Key
            let key = match self.expect("object key")? {
                Token::String(s) => s,
                found => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "object key",
                        found,
                        position: self.pos.saturating_sub(1),
                    });
                }
            };

            // consume the separator: colon
            match self.expect("colon") {
                Ok(Token::Colon) => {}
                Ok(found) => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "colon",
                        found,
                        position: self.pos.saturating_sub(1),
                    });
                }
                Err(error) => return Err(error),
            }

            let value = self.parse_value()?;

            pairs.push((key, value));

            // after a pair expected token should be a comma or right brace
            match self.current()? {
                Token::Comma => {
                    self.advance();
                }
                Token::RightBrace => {
                    self.advance();
                    break; // end of the object
                }
                found => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "',' or '}'",
                        found: found.clone(),
                        position: self.pos,
                    });
                }
            }
        }

        Ok(JsonValue::Object(pairs))
    }
}

/// Parses a complete JSON value from a token stream.
///
/// # Errors
///
/// Returns parser errors for malformed input and
/// [`JsonError::TrailingTokens`] when tokens remain after the top-level JSON
/// value.
pub fn parse(tokens: Vec<Token>) -> Result<JsonValue> {
    let mut parser = Parser::new(tokens);
    let value = parser.parse_value()?;

    if parser.pos < parser.tokens.len() {
        return Err(JsonError::TrailingTokens {
            found: parser.tokens[parser.pos].clone(),
            position: parser.pos,
        });
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_value – primitives ──────────────────────────────────

    #[test]
    fn test_parse_true() {
        let tokens = vec![Token::True];
        assert_eq!(parse(tokens).unwrap(), JsonValue::Boolean(true));
    }

    #[test]
    fn test_parse_false() {
        let tokens = vec![Token::False];
        assert_eq!(parse(tokens).unwrap(), JsonValue::Boolean(false));
    }

    #[test]
    fn test_parse_null() {
        let tokens = vec![Token::Null];
        assert_eq!(parse(tokens).unwrap(), JsonValue::Null);
    }

    #[test]
    fn test_parse_number() {
        let tokens = vec![Token::Number(3.14)];
        assert_eq!(parse(tokens).unwrap(), JsonValue::Number(3.14));
    }

    #[test]
    fn test_parse_number_negative() {
        let tokens = vec![Token::Number(-42.0)];
        assert_eq!(parse(tokens).unwrap(), JsonValue::Number(-42.0));
    }

    #[test]
    fn test_parse_string() {
        let tokens = vec![Token::String("hello".to_string())];
        assert_eq!(
            parse(tokens).unwrap(),
            JsonValue::String("hello".to_string())
        );
    }

    #[test]
    fn test_parse_string_empty() {
        let tokens = vec![Token::String("".to_string())];
        assert_eq!(parse(tokens).unwrap(), JsonValue::String("".to_string()));
    }

    // ── parse – empty containers ──────────────────────────────────

    #[test]
    fn test_parse_empty_object() {
        let tokens = vec![Token::LeftBrace, Token::RightBrace];
        assert_eq!(parse(tokens).unwrap(), JsonValue::Object(vec![]));
    }

    #[test]
    fn test_parse_empty_array() {
        let tokens = vec![Token::LeftBracket, Token::RightBracket];
        assert_eq!(parse(tokens).unwrap(), JsonValue::Array(vec![]));
    }

    // ── parse – objects ───────────────────────────────────────────

    #[test]
    fn test_parse_single_pair_object() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("key".to_string()),
            Token::Colon,
            Token::String("value".to_string()),
            Token::RightBrace,
        ];
        assert_eq!(
            parse(tokens).unwrap(),
            JsonValue::Object(vec![(
                "key".to_string(),
                JsonValue::String("value".to_string())
            )])
        );
    }

    #[test]
    fn test_parse_object_multiple_pairs() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("a".to_string()),
            Token::Colon,
            Token::Number(1.0),
            Token::Comma,
            Token::String("b".to_string()),
            Token::Colon,
            Token::Number(2.0),
            Token::Comma,
            Token::String("c".to_string()),
            Token::Colon,
            Token::True,
            Token::RightBrace,
        ];
        let expected = JsonValue::Object(vec![
            ("a".to_string(), JsonValue::Number(1.0)),
            ("b".to_string(), JsonValue::Number(2.0)),
            ("c".to_string(), JsonValue::Boolean(true)),
        ]);
        assert_eq!(parse(tokens).unwrap(), expected);
    }

    #[test]
    fn test_parse_nested_object() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("outer".to_string()),
            Token::Colon,
            Token::LeftBrace,
            Token::String("inner".to_string()),
            Token::Colon,
            Token::String("val".to_string()),
            Token::RightBrace,
            Token::RightBrace,
        ];
        let expected = JsonValue::Object(vec![(
            "outer".to_string(),
            JsonValue::Object(vec![(
                "inner".to_string(),
                JsonValue::String("val".to_string()),
            )]),
        )]);
        assert_eq!(parse(tokens).unwrap(), expected);
    }

    // ── parse – arrays ────────────────────────────────────────────

    #[test]
    fn test_parse_single_element_array() {
        let tokens = vec![Token::LeftBracket, Token::Number(1.0), Token::RightBracket];
        assert_eq!(
            parse(tokens).unwrap(),
            JsonValue::Array(vec![JsonValue::Number(1.0)])
        );
    }

    #[test]
    fn test_parse_array_multiple_elements() {
        let tokens = vec![
            Token::LeftBracket,
            Token::Number(1.0),
            Token::Comma,
            Token::String("two".to_string()),
            Token::Comma,
            Token::True,
            Token::Comma,
            Token::Null,
            Token::RightBracket,
        ];
        assert_eq!(
            parse(tokens).unwrap(),
            JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::String("two".to_string()),
                JsonValue::Boolean(true),
                JsonValue::Null,
            ])
        );
    }

    #[test]
    fn test_parse_nested_array() {
        let tokens = vec![
            Token::LeftBracket,
            Token::LeftBracket,
            Token::Number(1.0),
            Token::RightBracket,
            Token::Comma,
            Token::LeftBracket,
            Token::Number(2.0),
            Token::RightBracket,
            Token::RightBracket,
        ];
        assert_eq!(
            parse(tokens).unwrap(),
            JsonValue::Array(vec![
                JsonValue::Array(vec![JsonValue::Number(1.0)]),
                JsonValue::Array(vec![JsonValue::Number(2.0)]),
            ])
        );
    }

    // ── parse – mixed nesting ─────────────────────────────────────

    #[test]
    fn test_parse_array_of_objects() {
        let tokens = vec![
            Token::LeftBracket,
            Token::LeftBrace,
            Token::String("x".to_string()),
            Token::Colon,
            Token::Number(1.0),
            Token::RightBrace,
            Token::Comma,
            Token::LeftBrace,
            Token::String("y".to_string()),
            Token::Colon,
            Token::Number(2.0),
            Token::RightBrace,
            Token::RightBracket,
        ];
        let expected = JsonValue::Array(vec![
            JsonValue::Object(vec![("x".to_string(), JsonValue::Number(1.0))]),
            JsonValue::Object(vec![("y".to_string(), JsonValue::Number(2.0))]),
        ]);
        assert_eq!(parse(tokens).unwrap(), expected);
    }

    #[test]
    fn test_parse_object_with_array_values() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("nums".to_string()),
            Token::Colon,
            Token::LeftBracket,
            Token::Number(1.0),
            Token::Comma,
            Token::Number(2.0),
            Token::RightBracket,
            Token::RightBrace,
        ];
        let expected = JsonValue::Object(vec![(
            "nums".to_string(),
            JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]),
        )]);
        assert_eq!(parse(tokens).unwrap(), expected);
    }

    // ── parse – panic: object errors ──────────────────────────────

    #[test]
    #[should_panic]
    fn test_parse_object_missing_colon() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("key".to_string()),
            Token::String("val".to_string()),
            Token::RightBrace,
        ];
        parse(tokens).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_parse_object_non_string_key() {
        let tokens = vec![
            Token::LeftBrace,
            Token::Number(1.0),
            Token::Colon,
            Token::String("val".to_string()),
            Token::RightBrace,
        ];
        parse(tokens).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_parse_object_extra_comma() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("a".to_string()),
            Token::Colon,
            Token::Number(1.0),
            Token::Comma,
            Token::Comma,
            Token::String("b".to_string()),
            Token::Colon,
            Token::Number(2.0),
            Token::RightBrace,
        ];
        parse(tokens).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_parse_object_trailing_comma() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("a".to_string()),
            Token::Colon,
            Token::Number(1.0),
            Token::Comma,
            Token::RightBrace,
        ];
        parse(tokens).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_parse_object_missing_comma_between_pairs() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("a".to_string()),
            Token::Colon,
            Token::Number(1.0),
            Token::String("b".to_string()),
            Token::Colon,
            Token::Number(2.0),
            Token::RightBrace,
        ];
        parse(tokens).unwrap();
    }

    // ── parse – panic: array errors ───────────────────────────────

    #[test]
    #[should_panic]
    fn test_parse_array_extra_comma() {
        let tokens = vec![
            Token::LeftBracket,
            Token::Number(1.0),
            Token::Comma,
            Token::Comma,
            Token::Number(2.0),
            Token::RightBracket,
        ];
        parse(tokens).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_parse_array_trailing_comma() {
        let tokens = vec![
            Token::LeftBracket,
            Token::Number(1.0),
            Token::Comma,
            Token::RightBracket,
        ];
        parse(tokens).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_parse_array_missing_comma() {
        let tokens = vec![
            Token::LeftBracket,
            Token::Number(1.0),
            Token::Number(2.0),
            Token::RightBracket,
        ];
        parse(tokens).unwrap();
    }

    // ── parse – panic: unexpected tokens ──────────────────────────

    #[test]
    #[should_panic]
    fn test_parse_empty_input() {
        parse(vec![]).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_parse_unclosed_object() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("key".to_string()),
            Token::Colon,
            Token::Number(1.0),
            // missing RightBrace
        ];
        parse(tokens).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_parse_unclosed_array() {
        let tokens = vec![
            Token::LeftBracket,
            Token::Number(1.0),
            // missing RightBracket
        ];
        parse(tokens).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_parse_unexpected_right_brace() {
        let tokens = vec![Token::RightBrace];
        parse(tokens).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_parse_unexpected_right_bracket() {
        let tokens = vec![Token::RightBracket];
        parse(tokens).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_parse_unexpected_colon() {
        let tokens = vec![Token::Colon];
        parse(tokens).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_parse_unexpected_comma() {
        let tokens = vec![Token::Comma];
        parse(tokens).unwrap();
    }
}
