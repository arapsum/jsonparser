use std::{error::Error, fmt};

use crate::Token;

/// Result type used by the lexer and parser.
pub type Result<T> = std::result::Result<T, JsonError>;

/// Errors that can occur while tokenising or parsing JSON.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonError {
    /// The input ended before the expected item was found.
    UnexpectedEnd {
        /// Description of the item that was expected.
        expected: &'static str,
        /// Byte position or token position where the end was reached.
        position: usize,
    },
    /// A token was found where a different token or value was required.
    UnexpectedToken {
        /// Description of the token or value that was expected.
        expected: &'static str,
        /// The token that was actually found.
        found: Token,
        /// Token position where the unexpected token was found.
        position: usize,
    },
    /// A byte in the input is not valid at the current lexer position.
    UnexpectedCharacter {
        /// The unexpected character.
        character: char,
        /// Byte position of the unexpected character.
        position: usize,
    },
    /// A string escape sequence is not one of the supported JSON escapes.
    InvalidEscape {
        /// The escaped character after the backslash.
        character: char,
        /// Byte position of the invalid escaped character.
        position: usize,
    },
    /// A quoted string reached the end of input before a closing quote.
    UnterminatedString {
        /// Byte position where the string started.
        position: usize,
    },
    /// A string ended immediately after a backslash escape marker.
    UnterminatedEscape {
        /// Byte position where the escape sequence ended unexpectedly.
        position: usize,
    },
    /// A unicode escape contains a non-hexadecimal digit.
    InvalidUnicodeEscape {
        /// The invalid hexadecimal digit.
        character: char,
        /// Byte position of the invalid digit.
        position: usize,
    },
    /// A unicode escape ended before four hexadecimal digits were read.
    UnterminatedUnicodeEscape {
        /// Byte position where the unicode escape ended unexpectedly.
        position: usize,
    },
    /// A unicode escape decoded to a value that is not a valid scalar value.
    InvalidUnicodeCodepoint {
        /// The decoded codepoint value.
        codepoint: u32,
        /// Byte position after the unicode escape was read.
        position: usize,
    },
    /// A numeric literal could not be parsed as an `f64`.
    InvalidNumber {
        /// The numeric text that failed to parse.
        value: String,
        /// Byte position where the numeric literal started.
        position: usize,
    },
    /// Extra tokens remained after a complete top-level JSON value.
    TrailingTokens {
        /// The first extra token after the parsed JSON value.
        found: Token,
        /// Token position of the trailing token.
        position: usize,
    },
}

impl fmt::Display for JsonError {
    /// Formats a human-readable error message.
    ///
    /// # Errors
    ///
    /// Returns any formatting error reported by the formatter.
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
