use std::{
    any::{Any, TypeId},
    collections::HashMap,
    path::Path,
};

use anyhow::{anyhow, Result};

use crate::state::components::{ComponentId, ComponentsRef};

use super::{
    Config, ConfigId, ConfigIndexer, ConfigIndexerMap, ConfigLabel, ConfigRepositoryBuilder,
    ConfigTypeRegistry,
};

pub(super) type AnySendSync = dyn Any + Send + Sync;

#[repr(transparent)]
pub struct ConfigArray<C>(pub(super) Vec<C>);

pub struct ConfigStorageRef<'a, C>(&'a ConfigArray<C>, &'a ConfigIndexer);

pub struct ConfigStorageRefMut<'a, C>(&'a mut ConfigArray<C>, &'a mut ConfigIndexer);

pub struct ConfigRepository {
    /// (C : TypeId) -> ConfigArray<C> * ConfigIndexer
    pub(super) configs: HashMap<TypeId, (Box<AnySendSync>, ConfigIndexer)>,
}

impl ConfigRepository {
    pub fn new(
        cfg_ty_reg: &ConfigTypeRegistry,
        comps: ComponentsRef<'_>,
        comp_id: ComponentId,
    ) -> Result<Self> {
        ConfigRepositoryBuilder::new(cfg_ty_reg)?.build(cfg_ty_reg, comps, comp_id)
    }

    pub fn from_directory(
        cfg_ty_reg: &ConfigTypeRegistry,
        comps: ComponentsRef<'_>,
        comp_id: ComponentId,
        directory: &Path,
    ) -> Result<Self> {
        let mut builder = ConfigRepositoryBuilder::new(cfg_ty_reg)?;
        builder.load_directory(cfg_ty_reg, directory)?;
        builder.build(cfg_ty_reg, comps, comp_id)
    }

    pub fn get_indexer<C: Config>(&self) -> Result<&ConfigIndexer> {
        Ok(&self
            .configs
            .get(&TypeId::of::<C>())
            .ok_or_else(|| anyhow!("Config storage not registered: {}", C::TAG))?
            .1)
    }

    pub fn get_id_from_raw<C: Config>(&self, label: &str) -> Result<ConfigId<C>> {
        self.configs.get_id_from_raw(label)
    }

    pub fn get_label<C: Config>(&self, id: ConfigId<C>) -> Result<&ConfigLabel<C>> {
        self.configs.get_label(id)
    }

    pub fn get_storage<C: Config>(&self) -> Result<ConfigStorageRef<'_, C>> {
        let any_store = self
            .configs
            .get(&TypeId::of::<C>())
            .ok_or_else(|| anyhow!("Config storage not registered: {}", C::TAG))?;
        let storage = any_store
            .0
            .downcast_ref()
            .ok_or_else(|| anyhow!("Config storage had type not matching its key: {}", C::TAG))?;
        Ok(ConfigStorageRef(storage, &any_store.1))
    }

    pub fn get_storage_mut<C: Config>(&mut self) -> Result<ConfigStorageRefMut<'_, C>> {
        let any_store = self
            .configs
            .get_mut(&TypeId::of::<C>())
            .ok_or_else(|| anyhow!("Store not registered: {}", C::TAG))?;
        let storage = any_store
            .0
            .downcast_mut()
            .ok_or_else(|| anyhow!("Storage had type not matching its key: {}", C::TAG))?;
        Ok(ConfigStorageRefMut(storage, &mut any_store.1))
    }
}

impl<'a, C: Config> ConfigStorageRef<'a, C> {
    pub fn get(&self, id: ConfigId<C>) -> Result<&'a C> {
        let index: usize = id.0.try_into().unwrap();
        self.0 .0.get(index).ok_or(anyhow!(
            "Key '{}' doesn't exist in the store for config '{}'",
            self.1.report_id(id),
            C::TAG
        ))
    }

    pub fn configs(&self) -> impl Iterator<Item = &'a C> {
        self.0 .0.iter()
    }

    pub fn iter(&self) -> impl Iterator<Item = (ConfigId<C>, &'a C)> {
        self.configs()
            .enumerate()
            .map(|(i, c)| (ConfigId::new(i.try_into().unwrap()), c))
    }
}

struct ConfigStorageIterMut<'a, C>(&'a mut ConfigArray<C>, usize);

impl<'a, C> Iterator for ConfigStorageIterMut<'a, C> {
    type Item = (ConfigId<C>, &'a mut C);

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 >= self.0 .0.len() {
            return None;
        }
        let idx = self.1;
        self.1 += 1;
        let cfg = unsafe { &mut *((&mut self.0 .0[idx]) as *mut _) };
        Some((ConfigId::new(idx.try_into().unwrap()), cfg))
    }
}

impl<'a, C: Config> ConfigStorageRefMut<'a, C> {
    pub fn get_mut(&'a mut self, id: ConfigId<C>) -> Result<&'a mut C> {
        let index: usize = id.0.try_into().unwrap();
        self.0 .0.get_mut(index).ok_or(anyhow!(
            "Key '{}' doesn't exist in the store for config '{}'",
            self.1.report_id(id),
            C::TAG
        ))
    }

    pub fn into_mut(self, id: ConfigId<C>) -> Result<&'a mut C> {
        let index: usize = id.0.try_into().unwrap();
        self.0 .0.get_mut(index).ok_or(anyhow!(
            "Key '{}' doesn't exist in the store for config '{}'",
            self.1.report_id(id),
            C::TAG
        ))
    }

    pub fn configs_mut(&'a mut self) -> impl Iterator<Item = &'a mut C> {
        self.0 .0.iter_mut()
    }

    pub fn iter_mut(&'a mut self) -> impl Iterator<Item = (ConfigId<C>, &'a mut C)> {
        self.configs_mut()
            .enumerate()
            .map(|(i, c)| (ConfigId::new(i.try_into().unwrap()), c))
    }

    pub fn into_iter(self) -> impl Iterator<Item = (ConfigId<C>, &'a mut C)> {
        ConfigStorageIterMut(self.0, 0)
    }
}
