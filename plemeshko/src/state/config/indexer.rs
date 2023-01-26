use std::{
    any::{type_name, TypeId},
    borrow::Cow,
    collections::HashMap,
};

use anyhow::{anyhow, Result};
use educe::Educe;

use crate::state::indexer::Indexer;

use super::{Config, ConfigId, ConfigLabel, RawConfigId};

#[derive(Educe)]
#[educe(Default)]
#[repr(transparent)]
pub struct ConfigIndexer(Indexer<String, RawConfigId>);

impl ConfigIndexer {
    pub fn new() -> Self {
        ConfigIndexer::default()
    }

    pub fn get_id_from_raw<C: Config>(&self, label: &str) -> Result<ConfigId<C>> {
        Ok(ConfigId::new(self.0.get_id(label)?))
    }

    pub fn get_or_create_id<C: Config>(&mut self, label: Cow<'_, ConfigLabel<C>>) -> ConfigId<C> {
        ConfigId::new(self.0.get_or_create_id(match label {
            Cow::Borrowed(label) => Cow::Borrowed(&label.0),
            Cow::Owned(label) => Cow::Owned(label.0),
        }))
    }

    pub fn get_label<C: Config>(&self, id: ConfigId<C>) -> Result<&ConfigLabel<C>> {
        self.0
            .get_label(id.0)
            .map(|label| unsafe { std::mem::transmute(label) })
    }
}

pub trait ConfigIndexerMap {
    fn get_or_create_id<C: Config>(
        &mut self,
        label: Cow<'_, ConfigLabel<C>>,
    ) -> Result<ConfigId<C>>;
    fn get_id_from_raw<C: Config>(&self, label: &str) -> Result<ConfigId<C>>;
    fn get_label<C: Config>(&self, id: ConfigId<C>) -> Result<&ConfigLabel<C>>;
}

impl<T> ConfigIndexerMap for HashMap<TypeId, (T, ConfigIndexer)> {
    fn get_or_create_id<C: Config>(
        &mut self,
        label: Cow<'_, ConfigLabel<C>>,
    ) -> Result<ConfigId<C>> {
        Ok(self
            .get_mut(&TypeId::of::<C>())
            .ok_or_else(|| anyhow!("Storage not found for config type: {}", type_name::<C>()))?
            .1
            .get_or_create_id(label))
    }

    fn get_id_from_raw<C: Config>(&self, label: &str) -> Result<ConfigId<C>> {
        self.get(&TypeId::of::<C>())
            .ok_or_else(|| anyhow!("Storage not found for config type: {}", type_name::<C>()))?
            .1
            .get_id_from_raw(label)
    }

    fn get_label<C: Config>(&self, id: ConfigId<C>) -> Result<&ConfigLabel<C>> {
        self.get(&TypeId::of::<C>())
            .ok_or_else(|| anyhow!("Storage not found for config type: {}", type_name::<C>()))?
            .1
            .get_label(id)
    }
}
