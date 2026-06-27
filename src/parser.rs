use crate::{JsonValue, Token};

#[derive(Debug, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    #[must_use]
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    #[must_use]
    pub fn current(&self) -> &Token {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            panic!("Unexpected end of token stream");
        }
    }

    pub fn advance(&mut self) {
        self.pos += 1;
    }

    pub fn expect(&mut self, description: &str) -> &Token {
        if self.pos < self.tokens.len() {
            let token = &self.tokens[self.pos];
            self.pos += 1;
            token
        } else {
            panic!("Expected {description} but reached end of input");
        }
    }

    pub fn parse_value(&mut self) -> JsonValue {
        match self.current() {
            Token::LeftBrace => self.parse_object(),
            Token::LeftBracket => self.parse_array(),
            Token::True => {
                self.advance();
                JsonValue::Boolean(true)
            }
            Token::False => {
                self.advance();
                JsonValue::Boolean(false)
            }
            Token::Null => {
                self.advance();
                JsonValue::Null
            }
            Token::Number(n) => {
                let val = *n;
                self.advance();
                JsonValue::Number(val)
            }
            Token::String(s) => {
                let val = s.clone();
                self.advance();
                JsonValue::String(val)
            }
            Token::RightBrace => panic!("Unexpected '}}'"),
            Token::RightBracket => panic!("Unexpected ']'"),
            Token::Colon => panic!("Unexpected ':'"),
            Token::Comma => panic!("Unexpected ','"),
        }
    }

    pub fn parse_array(&mut self) -> JsonValue {
        self.advance(); // consume the opening paranthesis

        let mut array = Vec::new();
        // empty array
        if let Token::RightBracket = self.current() {
            self.advance();
            return JsonValue::Array(array);
        }

        loop {
            let element = self.parse_value(); // recursive call
            array.push(element);

            match self.current() {
                Token::Comma => {
                    self.advance(); // consume comma, seek next el
                }
                Token::RightBracket => {
                    self.advance(); // consume closing paranthesis, we are at the end
                    break;
                }
                _ => panic!("Expected ',' or ']' in array "),
            }
        }

        JsonValue::Array(array)
    }
    pub fn parse_object(&mut self) -> JsonValue {
        self.advance();

        let mut pairs: Vec<(String, JsonValue)> = Vec::new();

        // object may be empty
        if let Token::RightBrace = self.current() {
            self.advance();
            return JsonValue::Object(pairs);
        }

        loop {
            // parse the Key
            let key = match self.expect("object key") {
                Token::String(s) => s.clone(),
                other => panic!("Expected String key, got something else: {other:?}"),
            };

            // consume the separator: colon
            match self.expect("colon") {
                Token::Colon => {}
                _ => panic!("Expected ':' after object key"),
            }

            let value = self.parse_value();

            pairs.push((key, value));

            // after a pair expected token should be a comma or right brace
            match self.current() {
                Token::Comma => {
                    self.advance();
                }
                Token::RightBrace => {
                    self.advance();
                    break; // end of the object
                }
                _ => panic!("Expected ',' or '}}' in Object "),
            }
        }

        JsonValue::Object(pairs)
    }
}

#[must_use]
pub fn parse(tokens: Vec<Token>) -> JsonValue {
    let mut parser = Parser::new(tokens);
    parser.parse_value()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_value – primitives ──────────────────────────────────

    #[test]
    fn test_parse_true() {
        let tokens = vec![Token::True];
        assert_eq!(parse(tokens), JsonValue::Boolean(true));
    }

    #[test]
    fn test_parse_false() {
        let tokens = vec![Token::False];
        assert_eq!(parse(tokens), JsonValue::Boolean(false));
    }

    #[test]
    fn test_parse_null() {
        let tokens = vec![Token::Null];
        assert_eq!(parse(tokens), JsonValue::Null);
    }

    #[test]
    fn test_parse_number() {
        let tokens = vec![Token::Number(3.14)];
        assert_eq!(parse(tokens), JsonValue::Number(3.14));
    }

    #[test]
    fn test_parse_number_negative() {
        let tokens = vec![Token::Number(-42.0)];
        assert_eq!(parse(tokens), JsonValue::Number(-42.0));
    }

    #[test]
    fn test_parse_string() {
        let tokens = vec![Token::String("hello".to_string())];
        assert_eq!(parse(tokens), JsonValue::String("hello".to_string()));
    }

    #[test]
    fn test_parse_string_empty() {
        let tokens = vec![Token::String("".to_string())];
        assert_eq!(parse(tokens), JsonValue::String("".to_string()));
    }

    // ── parse – empty containers ──────────────────────────────────

    #[test]
    fn test_parse_empty_object() {
        let tokens = vec![Token::LeftBrace, Token::RightBrace];
        assert_eq!(parse(tokens), JsonValue::Object(vec![]));
    }

    #[test]
    fn test_parse_empty_array() {
        let tokens = vec![Token::LeftBracket, Token::RightBracket];
        assert_eq!(parse(tokens), JsonValue::Array(vec![]));
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
            parse(tokens),
            JsonValue::Object(vec![("key".to_string(), JsonValue::String("value".to_string()))])
        );
    }

    #[test]
    fn test_parse_object_multiple_pairs() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("a".to_string()), Token::Colon, Token::Number(1.0), Token::Comma,
            Token::String("b".to_string()), Token::Colon, Token::Number(2.0), Token::Comma,
            Token::String("c".to_string()), Token::Colon, Token::True,
            Token::RightBrace,
        ];
        let expected = JsonValue::Object(vec![
            ("a".to_string(), JsonValue::Number(1.0)),
            ("b".to_string(), JsonValue::Number(2.0)),
            ("c".to_string(), JsonValue::Boolean(true)),
        ]);
        assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn test_parse_nested_object() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("outer".to_string()), Token::Colon,
            Token::LeftBrace,
            Token::String("inner".to_string()), Token::Colon, Token::String("val".to_string()),
            Token::RightBrace,
            Token::RightBrace,
        ];
        let expected = JsonValue::Object(vec![
            ("outer".to_string(), JsonValue::Object(vec![
                ("inner".to_string(), JsonValue::String("val".to_string())),
            ])),
        ]);
        assert_eq!(parse(tokens), expected);
    }

    // ── parse – arrays ────────────────────────────────────────────

    #[test]
    fn test_parse_single_element_array() {
        let tokens = vec![
            Token::LeftBracket,
            Token::Number(1.0),
            Token::RightBracket,
        ];
        assert_eq!(parse(tokens), JsonValue::Array(vec![JsonValue::Number(1.0)]));
    }

    #[test]
    fn test_parse_array_multiple_elements() {
        let tokens = vec![
            Token::LeftBracket,
            Token::Number(1.0), Token::Comma,
            Token::String("two".to_string()), Token::Comma,
            Token::True, Token::Comma,
            Token::Null,
            Token::RightBracket,
        ];
        assert_eq!(
            parse(tokens),
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
            Token::LeftBracket, Token::Number(1.0), Token::RightBracket, Token::Comma,
            Token::LeftBracket, Token::Number(2.0), Token::RightBracket,
            Token::RightBracket,
        ];
        assert_eq!(
            parse(tokens),
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
            Token::LeftBrace, Token::String("x".to_string()), Token::Colon, Token::Number(1.0), Token::RightBrace, Token::Comma,
            Token::LeftBrace, Token::String("y".to_string()), Token::Colon, Token::Number(2.0), Token::RightBrace,
            Token::RightBracket,
        ];
        let expected = JsonValue::Array(vec![
            JsonValue::Object(vec![("x".to_string(), JsonValue::Number(1.0))]),
            JsonValue::Object(vec![("y".to_string(), JsonValue::Number(2.0))]),
        ]);
        assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn test_parse_object_with_array_values() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("nums".to_string()), Token::Colon,
            Token::LeftBracket, Token::Number(1.0), Token::Comma, Token::Number(2.0), Token::RightBracket,
            Token::RightBrace,
        ];
        let expected = JsonValue::Object(vec![
            ("nums".to_string(), JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
            ])),
        ]);
        assert_eq!(parse(tokens), expected);
    }

    // ── parse – panic: object errors ──────────────────────────────

    #[test]
    #[should_panic(expected = "Expected ':' after object key")]
    fn test_parse_object_missing_colon() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("key".to_string()),
            Token::String("val".to_string()),
            Token::RightBrace,
        ];
        parse(tokens);
    }

    #[test]
    #[should_panic(expected = "Expected String key")]
    fn test_parse_object_non_string_key() {
        let tokens = vec![
            Token::LeftBrace,
            Token::Number(1.0),
            Token::Colon,
            Token::String("val".to_string()),
            Token::RightBrace,
        ];
        parse(tokens);
    }

    #[test]
    #[should_panic(expected = "Expected String key")]
    fn test_parse_object_extra_comma() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("a".to_string()), Token::Colon, Token::Number(1.0), Token::Comma, Token::Comma,
            Token::String("b".to_string()), Token::Colon, Token::Number(2.0),
            Token::RightBrace,
        ];
        parse(tokens);
    }

    #[test]
    #[should_panic(expected = "Expected String key")]
    fn test_parse_object_trailing_comma() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("a".to_string()), Token::Colon, Token::Number(1.0), Token::Comma,
            Token::RightBrace,
        ];
        parse(tokens);
    }

    #[test]
    #[should_panic(expected = "Expected ',' or '}' in Object")]
    fn test_parse_object_missing_comma_between_pairs() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("a".to_string()), Token::Colon, Token::Number(1.0),
            Token::String("b".to_string()), Token::Colon, Token::Number(2.0),
            Token::RightBrace,
        ];
        parse(tokens);
    }

    // ── parse – panic: array errors ───────────────────────────────

    #[test]
    #[should_panic(expected = "Unexpected ','")]
    fn test_parse_array_extra_comma() {
        let tokens = vec![
            Token::LeftBracket,
            Token::Number(1.0), Token::Comma, Token::Comma,
            Token::Number(2.0),
            Token::RightBracket,
        ];
        parse(tokens);
    }

    #[test]
    #[should_panic(expected = "Unexpected ']'")]
    fn test_parse_array_trailing_comma() {
        let tokens = vec![
            Token::LeftBracket,
            Token::Number(1.0), Token::Comma,
            Token::RightBracket,
        ];
        parse(tokens);
    }

    #[test]
    #[should_panic(expected = "Expected ',' or ']' in array")]
    fn test_parse_array_missing_comma() {
        let tokens = vec![
            Token::LeftBracket,
            Token::Number(1.0),
            Token::Number(2.0),
            Token::RightBracket,
        ];
        parse(tokens);
    }

    // ── parse – panic: unexpected tokens ──────────────────────────

    #[test]
    #[should_panic(expected = "Unexpected end of token stream")]
    fn test_parse_empty_input() {
        parse(vec![]);
    }

    #[test]
    #[should_panic(expected = "Unexpected end")]
    fn test_parse_unclosed_object() {
        let tokens = vec![
            Token::LeftBrace,
            Token::String("key".to_string()), Token::Colon, Token::Number(1.0),
            // missing RightBrace
        ];
        parse(tokens);
    }

    #[test]
    #[should_panic(expected = "Unexpected end")]
    fn test_parse_unclosed_array() {
        let tokens = vec![
            Token::LeftBracket,
            Token::Number(1.0),
            // missing RightBracket
        ];
        parse(tokens);
    }

    #[test]
    #[should_panic(expected = "Unexpected '}'")]
    fn test_parse_unexpected_right_brace() {
        let tokens = vec![Token::RightBrace];
        parse(tokens);
    }

    #[test]
    #[should_panic(expected = "Unexpected ']'")]
    fn test_parse_unexpected_right_bracket() {
        let tokens = vec![Token::RightBracket];
        parse(tokens);
    }

    #[test]
    #[should_panic(expected = "Unexpected ':'")]
    fn test_parse_unexpected_colon() {
        let tokens = vec![Token::Colon];
        parse(tokens);
    }

    #[test]
    #[should_panic(expected = "Unexpected ','")]
    fn test_parse_unexpected_comma() {
        let tokens = vec![Token::Comma];
        parse(tokens);
    }
}
