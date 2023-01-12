use std::{error::Error, fmt::Display};

use plegine::config::ConfigRetrievalError;

#[derive(Debug)]
pub enum GuiError {
    ConfigRetrievalFailed(ConfigRetrievalError),
}

pub type GuiResult<T> = Result<T, GuiError>;

impl Display for GuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuiError::ConfigRetrievalFailed(e) => write!(f, "Config retrieval failed: {e}"),
        }
    }
}

impl Error for GuiError {}
