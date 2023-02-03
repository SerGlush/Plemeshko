use std::{
    any::{type_name, TypeId},
    borrow::Cow,
    collections::HashMap,
};

use anyhow::{anyhow, Result};
use bytemuck::TransparentWrapper;
use educe::Educe;

use crate::state::raw_indexer::RawIndexer;

use super::{Config, ConfigId, ConfigLabel, RawConfigId};

#[derive(Educe)]
#[educe(Default)]
#[repr(transparent)]
pub struct ConfigIndexer(RawIndexer<String, RawConfigId>);

// todo: prevent usage of incompatible `C` for ConfigIndexer (parametrize by C)
impl ConfigIndexer {
    pub fn new() -> Self {
        ConfigIndexer::default()
    }

    pub fn id<C: Config>(&self, label: &ConfigLabel<C>) -> Result<ConfigId<C>> {
        self.id_from_raw(&label.0)
    }

    pub fn id_from_raw<C: Config>(&self, label: &str) -> Result<ConfigId<C>> {
        Ok(ConfigId::new(self.0.id(label)?))
    }

    pub fn declare_id<C: Config>(&mut self, label: Cow<'_, ConfigLabel<C>>) -> ConfigId<C> {
        ConfigId::new(self.0.declare_id(match label {
            Cow::Borrowed(label) => Cow::Borrowed(&label.0),
            Cow::Owned(label) => Cow::Owned(label.0),
        }))
    }

    pub fn label<C: Config>(&self, id: ConfigId<C>) -> Result<&ConfigLabel<C>> {
        self.0.label(id.0).map(ConfigLabel::wrap_ref)
    }

    pub fn indices<C: Config>(&self) -> impl Iterator<Item = ConfigId<C>> {
        (0..self.0.id_to_label.len()).map(|i| ConfigId::new(i.try_into().unwrap()))
    }

    pub(super) fn report_id<C: Config>(&self, id: ConfigId<C>) -> String {
        self.0.report_id(id.0)
    }
}

pub trait ConfigIndexerMap {
    fn declare_id<C: Config>(&mut self, label: Cow<'_, ConfigLabel<C>>) -> Result<ConfigId<C>>;
    fn id_from_raw<C: Config>(&self, label: &str) -> Result<ConfigId<C>>;
    fn label<C: Config>(&self, id: ConfigId<C>) -> Result<&ConfigLabel<C>>;
}

impl<T> ConfigIndexerMap for HashMap<TypeId, (T, ConfigIndexer)> {
    fn declare_id<C: Config>(&mut self, label: Cow<'_, ConfigLabel<C>>) -> Result<ConfigId<C>> {
        Ok(self
            .get_mut(&TypeId::of::<C>())
            .ok_or_else(|| anyhow!("Storage not found for config type: {}", type_name::<C>()))?
            .1
            .declare_id(label))
    }

    fn id_from_raw<C: Config>(&self, label: &str) -> Result<ConfigId<C>> {
        self.get(&TypeId::of::<C>())
            .ok_or_else(|| anyhow!("Storage not found for config type: {}", type_name::<C>()))?
            .1
            .id_from_raw(label)
    }

    fn label<C: Config>(&self, id: ConfigId<C>) -> Result<&ConfigLabel<C>> {
        self.get(&TypeId::of::<C>())
            .ok_or_else(|| anyhow!("Storage not found for config type: {}", type_name::<C>()))?
            .1
            .label(id)
    }
}
