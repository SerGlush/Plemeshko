mod repository;
mod repository_builder;

pub use repository::*;
pub use repository_builder::*;

use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, fmt::Display, marker::PhantomData};

pub trait Config: for<'a> Deserialize<'a> + Sized + Send + Sync + 'static {
    const TAG: &'static str;
}

#[derive(Educe, Serialize, Deserialize)]
#[educe(Clone, Hash, PartialEq, Eq, Debug)]
#[serde(transparent)]
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
