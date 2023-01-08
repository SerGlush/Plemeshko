use std::fmt::Display;

use plegine::config::ConfigRetrievalError;

pub enum SimError {
    ConfigRetrievalFailed(ConfigRetrievalError),
}

pub type SimResult<T> = Result<T, SimError>;

impl Display for SimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
