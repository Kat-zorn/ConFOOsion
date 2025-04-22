use std::fmt::Display;

use crate::PutBackChars;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub comment: String,
    line: usize,
    column: usize,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}. {}", self.line, self.column, self.comment)
    }
}

impl ParseError {
    pub fn from_str(chars: &PutBackChars, message: &'static str) -> Self {
        Self {
            comment: message.to_string(),
            line: chars.line_number,
            column: chars.column_number,
        }
    }
    pub fn from_string(chars: &PutBackChars, message: String) -> Self {
        Self {
            comment: message,
            line: chars.line_number,
            column: chars.column_number,
        }
    }
    pub fn empty(message: &str) -> Self {
        Self {
            comment: message.to_string(),
            line: 0,
            column: 0,
        }
    }
}
