use crate::{JsonValue, tokeniser::Token};

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
