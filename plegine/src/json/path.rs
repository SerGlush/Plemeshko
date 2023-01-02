use std::fmt::Display;

#[derive(Debug)]
pub struct Path(String);

impl Path {
    pub const DELIMETER: char = '/';

    fn check_key(key: &str) {
        if key.contains(Self::DELIMETER) {
            panic!("Json path key {key} contains delimeters.");
        }
    }

    /// Constructs path from a single key.
    /// Panics when key contains delimeters.
    pub fn from_key(key: String) -> Path {
        Self::check_key(&key);
        Path(key)
    }

    /// Constructs empty path. "self"
    pub fn new() -> Path {
        Path(String::new())
    }

    /// Prepends a key.
    /// Panics when key contains delimeters.
    pub fn lift(mut self, key: &str) -> Path {
        Self::check_key(key);
        self.0
            .reserve(self.0.len() + key.len() + Self::DELIMETER.len_utf8());
        self.0.insert_str(0, key);
        self.0.insert(0, Self::DELIMETER);
        self
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
