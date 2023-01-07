use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum GuiError {}

pub type GuiResult<T> = Result<T, GuiError>;

impl Display for GuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for GuiError {}
