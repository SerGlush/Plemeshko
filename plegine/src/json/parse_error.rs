use std::{error::Error, fmt::Display};

use super::path::Path;

#[derive(Debug)]
pub enum ParseErrorKind {
    UnexpectedType,
    ValidationFailed,
    FieldAbsent,
}

#[derive(Debug)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub path: Path,
    pub expected: String,
}

pub type ParseResult<T> = Result<T, ParseError>;

impl ParseError {
    pub fn lift(self, key: &str) -> Self {
        ParseError {
            path: self.path.lift(key),
            ..self
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ParseErrorKind::UnexpectedType => write!(f, "Unexpected type, "),
            ParseErrorKind::ValidationFailed => write!(f, "Validation failed, "),
            ParseErrorKind::FieldAbsent => write!(f, "Field absent, "),
        }?;
        write!(f, "expected \"{}\", ", self.expected)?;
        write!(f, "path \"{}\"", self.path)
    }
}

impl Error for ParseError {}
