mod path;
mod parse_error;
mod from_value;

pub use from_value::*;
pub use parse_error::*;
pub use path::*;
pub use serde_json::{Number, Value};

pub type Null = ();
pub type Array = Vec<Value>;
pub type Object = serde_json::Map<String, Value>;
