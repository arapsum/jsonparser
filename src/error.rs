use std::{error::Error, fmt};

use crate::Token;

pub type Result<T> = std::result::Result<T, JsonError>;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonError {
    UnexpectedEnd {
        expected: &'static str,
        position: usize,
    },
    UnexpectedToken {
        expected: &'static str,
        found: Token,
        position: usize,
    },
    UnexpectedCharacter {
        character: char,
        position: usize,
    },
    InvalidEscape {
        character: char,
        position: usize,
    },
    UnterminatedString {
        position: usize,
    },
    UnterminatedEscape {
        position: usize,
    },
    InvalidUnicodeEscape {
        character: char,
        position: usize,
    },
    UnterminatedUnicodeEscape {
        position: usize,
    },
    InvalidUnicodeCodepoint {
        codepoint: u32,
        position: usize,
    },
    InvalidNumber {
        value: String,
        position: usize,
    },
    TrailingTokens {
        found: Token,
        position: usize,
    },
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd { expected, position } => {
                write!(
                    f,
                    "expected {expected} at position {position}, but reached end of input"
                )
            }
            Self::UnexpectedToken {
                expected,
                found,
                position,
            } => write!(
                f,
                "expected {expected} at position {position}, but found {found:?}"
            ),
            Self::UnexpectedCharacter {
                character,
                position,
            } => write!(
                f,
                "unexpected character '{character}' at position {position}"
            ),
            Self::InvalidEscape {
                character,
                position,
            } => write!(
                f,
                "invalid escape sequence '\\{character}' at position {position}"
            ),
            Self::UnterminatedString { position } => {
                write!(f, "unterminated string starting at position {position}")
            }
            Self::UnterminatedEscape { position } => {
                write!(f, "unterminated escape sequence at position {position}")
            }
            Self::InvalidUnicodeEscape {
                character,
                position,
            } => write!(
                f,
                "invalid hex digit '{character}' in unicode escape at position {position}"
            ),
            Self::UnterminatedUnicodeEscape { position } => {
                write!(f, "unterminated unicode escape at position {position}")
            }
            Self::InvalidUnicodeCodepoint {
                codepoint,
                position,
            } => write!(
                f,
                "invalid unicode codepoint {codepoint} at position {position}"
            ),
            Self::InvalidNumber { value, position } => {
                write!(f, "invalid number '{value}' at position {position}")
            }
            Self::TrailingTokens { found, position } => write!(
                f,
                "unexpected trailing token {found:?} at position {position}"
            ),
        }
    }
}

impl Error for JsonError {}
