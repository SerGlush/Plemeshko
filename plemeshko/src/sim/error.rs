use plegine::config::ConfigRetrievalError;

pub enum SimError {
    ConfigRetrievalFailed(ConfigRetrievalError),
}

pub type SimResult<T> = Result<T, SimError>;
