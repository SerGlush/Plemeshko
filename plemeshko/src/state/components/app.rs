use std::borrow::Cow;

use anyhow::{anyhow, Result};
use colored::Colorize;
use fluent::FluentArgs;

use crate::state::{
    text::{TextIdRef, TextRepository, TextRetrievalError},
    texture::TextureRepository,
};

use super::ComponentId;

pub struct AppComponent {
    pub texts: TextRepository,
    pub textures: TextureRepository,
}

#[derive(Default)]
pub struct AppComponents(pub(super) Vec<Option<AppComponent>>);

// todo: return TextId prefixed with ComponentLabel When text is not found
// (block: `text` method is in components, which don't know their names)
impl AppComponent {
    pub fn text<'a>(
        &'a self,
        text_id: &TextIdRef,
        args: Option<&'a FluentArgs<'_>>,
    ) -> Result<Cow<'a, str>> {
        self.texts.get(text_id, args).or_else(|e| match e {
            TextRetrievalError::NotFound(id) => {
                log::warn!(
                    "Text retrieval failed for:\n{}",
                    crate::log::colorize(&id, Colorize::magenta)
                );
                Ok(Cow::Owned(id))
            }
            e => Err(e.into()),
        })
    }
}

impl AppComponents {
    pub fn component(&self, id: ComponentId) -> Result<&AppComponent> {
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

    pub fn iter_components_mut(
        &mut self,
    ) -> impl Iterator<Item = (ComponentId, &mut AppComponent)> {
        self.0
            .iter_mut()
            .enumerate()
            .filter_map(|(id, c)| c.as_mut().map(|c| (ComponentId(id as u16), c)))
    }
}
