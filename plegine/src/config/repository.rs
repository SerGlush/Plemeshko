use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
};

use thiserror::Error;

use super::{Config, ConfigId};

pub(super) type HomoConfigContainer<C> = HashMap<String, C>;

pub trait AnyHomoConfigContainer = Any + Send + Sync;

pub struct ConfigRepository(pub(super) HashMap<TypeId, Box<dyn AnyHomoConfigContainer>>);

#[derive(Debug, Error)]
pub enum ConfigRetrievalError {
    #[error("Store not registered for '{type_name}'")]
    StoreNotRegistered { type_name: &'static str },
    #[error("Storage had type not matching its key, expected ('{type_name}')")]
    StoreTypeMismatch { type_name: &'static str },
    #[error("Key '{id}' doesn't exist in the store for config type '{type_name}'")]
    NotInStore { type_name: &'static str, id: String },
}

impl ConfigRepository {
    fn get_store<C: Config>(&self) -> Result<&HomoConfigContainer<C>, ConfigRetrievalError> {
        let any_store = self.0.get(&TypeId::of::<C>()).ok_or_else(|| {
            ConfigRetrievalError::StoreNotRegistered {
                type_name: type_name::<C>(),
            }
        })?;
        any_store
            .downcast_ref()
            .ok_or_else(|| ConfigRetrievalError::StoreTypeMismatch {
                type_name: type_name::<C>(),
            })
    }

    pub fn get<C: Config>(&self, id: &ConfigId<C>) -> Result<&C, ConfigRetrievalError> {
        match self.get_store::<C>() {
            Ok(store) => store
                .get(id.as_str())
                .ok_or(ConfigRetrievalError::NotInStore {
                    type_name: type_name::<C>(),
                    id: id.as_str().to_owned(),
                }),
            Err(e) => Err(e.into()),
        }
    }
}
