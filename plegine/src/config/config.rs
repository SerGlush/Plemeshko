use std::{borrow::Borrow, fmt::Display, hash::Hash, marker::PhantomData};

use crate::json::{self, FromValue};

pub type ConfigTag = &'static str;

// Reserved - can't be used for config's field names
pub const CONFIG_RESERVED_FIELD_TAG: &'static str = "#tag";
pub const CONFIG_RESERVED_FIELD_ID: &'static str = "#name";

pub trait Config: Sized + Send + 'static {
    const TAG: ConfigTag;

    fn parse(src: json::Object) -> Result<Self, json::ParseError>;
}

pub struct ConfigId<C>(String, PhantomData<C>);

impl<C> ConfigId<C> {
    pub fn new<S: Into<String>>(id: S) -> Self {
        ConfigId(id.into(), PhantomData)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl<C> Clone for ConfigId<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<C> Hash for ConfigId<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<C> PartialEq for ConfigId<C> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<C> Eq for ConfigId<C> {}

impl<C> FromValue for ConfigId<C> {
    fn from_value(value: serde_json::Value) -> json::ParseResult<Self> {
        Ok(ConfigId::new(String::from_value(value)?))
    }
}

impl<C> From<String> for ConfigId<C> {
    fn from(value: String) -> Self {
        ConfigId(value, PhantomData)
    }
}

impl<C> Display for ConfigId<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<C> Borrow<str> for ConfigId<C> {
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}
