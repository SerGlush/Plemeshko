use super::path::Path;

pub enum ParseErrorKind {
    UnexpectedType,
    ValidationFailed,
    FieldAbsent,
}

pub struct ParseError {
    pub kind: ParseErrorKind,
    pub path: Path,
    pub expected: String,
}

impl ParseError {
    pub fn lift(self, key: &str) -> Self {
        ParseError {
            path: self.path.lift(key),
            ..self
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
