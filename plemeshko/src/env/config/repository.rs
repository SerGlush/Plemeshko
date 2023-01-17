use std::{
    any::{Any, TypeId},
    borrow::Cow,
    collections::HashMap,
};

use anyhow::{anyhow, Result};

use super::{id::UnderId, Config, ConfigId, ConfigIndexer};

pub(super) type IdMap<C> = HashMap<UnderId, C>;

pub(super) type AnyIdMap = dyn Any + Send + Sync;

pub struct ConfigRepository {
    pub(super) configs: HashMap<TypeId, Box<AnyIdMap>>,
    pub indexer: ConfigIndexer,
}

impl ConfigRepository {
    fn get_store<C: Config>(&self) -> Result<&IdMap<C>> {
        let any_store = self
            .configs
            .get(&TypeId::of::<C>())
            .ok_or_else(|| anyhow!("Store not registered: {}", C::TAG))?;
        any_store
            .downcast_ref()
            .ok_or_else(|| anyhow!("Storage had type not matching its key: {}", C::TAG))
    }

    pub fn get<C: Config>(&self, id: ConfigId<C>) -> Result<&C> {
        match self.get_store::<C>() {
            Ok(store) => store.get(&id.0).ok_or(anyhow!(
                "Key '{}' doesn't exist in the store for config '{}'",
                self.indexer.get_label(id).map_or_else(
                    |_| Cow::Owned(id.0.to_string() + "?!"),
                    |label| Cow::Borrowed(&label.0)
                ),
                C::TAG
            )),
            Err(e) => Err(e),
        }
    }
}
