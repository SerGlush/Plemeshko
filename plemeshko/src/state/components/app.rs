use std::borrow::Cow;

use anyhow::{anyhow, Result};
use fluent::FluentArgs;

use crate::state::text::{TextIdentifier, TextRepository, TextRetrievalError};

use super::ComponentId;

pub struct AppComponent {
    pub texts: TextRepository,
}

#[derive(Default)]
pub struct AppComponents(pub(super) Vec<Option<AppComponent>>);

impl AppComponent {
    pub fn get_text<'a>(
        &'a self,
        text_id: &(impl TextIdentifier + ?Sized),
        args: Option<&'a FluentArgs<'_>>,
    ) -> Result<Cow<'a, str>> {
        self.texts.get(text_id, args).or_else(|e| match e {
            TextRetrievalError::NotFound(id) => Ok(Cow::Owned(id)),
            e => Err(e.into()),
        })
    }
}

impl AppComponents {
    pub fn get_component(&self, id: ComponentId) -> Result<&AppComponent> {
        self.0
            .get(id.0 as usize)
            .ok_or_else(|| anyhow!("Component id out of range: {}", id.0))?
            .as_ref()
            .ok_or_else(|| {
                anyhow!(
                    "Component hasn't finished loading or was unloaded: {}",
                    id.0
                )
            })
    }
}