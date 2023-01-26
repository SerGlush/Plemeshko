use std::{
    any::{Any, TypeId},
    borrow::Cow,
    collections::HashMap,
    path::Path,
};

use anyhow::{anyhow, Result};

use crate::state::components::ComponentLoadingContext;

use super::{
    Config, ConfigId, ConfigIndexer, ConfigIndexerMap, ConfigLabel, ConfigRepositoryBuilder,
    ConfigTypeRegistry,
};

pub(super) type AnySendSync = dyn Any + Send + Sync;

pub struct ConfigRepository {
    /// (C : TypeId) -> Vec<C> * ConfigIndexer
    pub(super) configs: HashMap<TypeId, (Box<AnySendSync>, ConfigIndexer)>,
}

impl ConfigRepository {
    pub fn new(
        cfg_ty_reg: &ConfigTypeRegistry,
        ctx: &mut ComponentLoadingContext<'_, ()>,
    ) -> Result<Self> {
        ConfigRepositoryBuilder::new(cfg_ty_reg)?.build(cfg_ty_reg, ctx)
    }

    pub fn from_directory(
        cfg_ty_reg: &ConfigTypeRegistry,
        ctx: &mut ComponentLoadingContext<'_, ()>,
        directory: &Path,
    ) -> Result<Self> {
        let mut builder = ConfigRepositoryBuilder::new(cfg_ty_reg)?;
        builder.load_directory(cfg_ty_reg, directory)?;
        builder.build(cfg_ty_reg, ctx)
    }

    pub fn get_storage<C: Config>(&self) -> Result<&Vec<C>> {
        let any_store = self
            .configs
            .get(&TypeId::of::<C>())
            .ok_or_else(|| anyhow!("Store not registered: {}", C::TAG))?;
        any_store
            .0
            .downcast_ref()
            .ok_or_else(|| anyhow!("Storage had type not matching its key: {}", C::TAG))
    }

    pub fn iter<C: Config>(&self) -> Result<impl Iterator<Item = (ConfigId<C>, &C)>> {
        Ok(self
            .get_storage()?
            .iter()
            .enumerate()
            .map(|(i, c)| (ConfigId::new(i.try_into().unwrap()), c)))
    }

    pub fn get<C: Config>(&self, id: ConfigId<C>) -> Result<&C> {
        self.get_storage::<C>().and_then(|store| {
            let index: usize = id.0.try_into().unwrap();
            store.get(index).ok_or(anyhow!(
                "Key '{}' doesn't exist in the store for config '{}'",
                self.configs.get_label(id).map_or_else(
                    |_| Cow::Owned(id.0.to_string() + "?!"),
                    |label| Cow::Borrowed(&label.0)
                ),
                C::TAG
            ))
        })
    }
}

impl ConfigIndexerMap for ConfigRepository {
    fn get_or_create_id<C: Config>(
        &mut self,
        label: Cow<'_, ConfigLabel<C>>,
    ) -> Result<ConfigId<C>> {
        self.configs.get_or_create_id(label)
    }

    fn get_id_from_raw<C: Config>(&self, label: &str) -> Result<ConfigId<C>> {
        self.configs.get_id_from_raw(label)
    }

    fn get_label<C: Config>(&self, id: ConfigId<C>) -> Result<&ConfigLabel<C>> {
        self.configs.get_label(id)
    }
}
