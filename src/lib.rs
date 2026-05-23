#[derive(Debug, Clone)]
pub enum JsonValue {
    Object(Vec<(String, JsonValue)>),
    Array(Vec<JsonValue>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone)]
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
    pub fn new(input: &str) -> Self {
        Self {
            input: input.as_bytes().to_vec(),
            pos: 0,
        }
    }

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
                        b'0'..=b'9' => (c - b'0') as u32,
                        b'a'..=b'f' => (c - b'a' + 10) as u32,
                        b'A'..=b'F' => (c - b'A' + 10) as u32,
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
            .unwrap_or_else(|_| panic!("Invalid number: {}", s))
    }
}

pub fn tokenise(input: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    loop {
        match lexer.current() {
            None => break,

            Some(b' ') | Some(b'\t') | Some(b'\r') | Some(b'\n') => {
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
            Some(b'-') | Some(b'0'..=b'9') => {
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

pub fn parse(tokens: &[Token]) -> JsonValue {
    todo!()
}
