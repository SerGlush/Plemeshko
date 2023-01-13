use std::{error::Error, fmt::Display};

use plegine::config::ConfigRetrievalError;

#[derive(Debug)]
pub enum AppError {
    ConfigRetrievalFailed(ConfigRetrievalError),
}

pub type AppResult<T> = Result<T, AppError>;

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::ConfigRetrievalFailed(e) => write!(f, "Config retrieval failed: {e}"),
        }
    }
}

impl Error for AppError {}
