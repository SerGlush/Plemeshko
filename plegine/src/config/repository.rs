use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
};

use super::{Config, ConfigId};

pub(super) type HomoConfigContainer<C> = HashMap<String, C>;

pub trait AnyHomoConfigContainer = Any + Send;

pub struct ConfigRepository(pub(super) HashMap<TypeId, Box<dyn AnyHomoConfigContainer>>);

pub enum ConfigStoreRetrievalError {
    StoreNotRegistered { kind: &'static str },
    StoreTypeInvalid,
}

pub enum ConfigRetrievalError {
    StoreNotRegistered { kind: &'static str },
    StoreTypeInvalid,
    NotInStore { kind: &'static str, id: String },
}

impl From<ConfigStoreRetrievalError> for ConfigRetrievalError {
    fn from(value: ConfigStoreRetrievalError) -> Self {
        match value {
            ConfigStoreRetrievalError::StoreNotRegistered { kind } => {
                ConfigRetrievalError::StoreNotRegistered { kind }
            }
            ConfigStoreRetrievalError::StoreTypeInvalid => ConfigRetrievalError::StoreTypeInvalid,
        }
    }
}

impl ConfigRepository {
    fn get_store<C: Config>(&self) -> Result<&HomoConfigContainer<C>, ConfigStoreRetrievalError> {
        let any_store = self.0.get(&TypeId::of::<C>()).ok_or(
            ConfigStoreRetrievalError::StoreNotRegistered {
                kind: type_name::<C>(),
            },
        )?;
        any_store
            .downcast_ref()
            .ok_or(ConfigStoreRetrievalError::StoreTypeInvalid)
    }

    pub fn get<C: Config>(&self, id: &ConfigId<C>) -> Result<&C, ConfigRetrievalError> {
        match self.get_store::<C>() {
            Ok(store) => store
                .get(id.as_str())
                .ok_or(ConfigRetrievalError::NotInStore {
                    kind: type_name::<C>(),
                    id: id.as_str().to_owned(),
                }),
            Err(e) => Err(e.into()),
        }
    }

    // pub fn check_id<C: Config>(&self, id: String) -> Option<ConfigId<C>> {
    //     let store = self.0.get(&TypeId::of::<C>())?;
    //     let hcc = store.downcast_ref::<HomoConfigContainer<C>>()?;
    //     if hcc.contains_key(id.as_str()) {
    //         Some(ConfigId::new(id))
    //     } else {
    //         None
    //     }
    // }
}
