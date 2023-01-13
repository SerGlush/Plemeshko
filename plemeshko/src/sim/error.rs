use std::{error::Error, fmt::Display};

use plegine::config::ConfigRetrievalError;

#[derive(Debug)]
pub enum SimError {
    ConfigRetrievalFailed(ConfigRetrievalError),
}

pub type SimResult<T> = Result<T, SimError>;

impl Display for SimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimError::ConfigRetrievalFailed(e) => write!(f, "Config retrieval failed: {e}"),
        }
    }
}

impl Error for SimError {}
