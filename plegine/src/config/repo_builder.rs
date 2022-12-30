use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::json::{self, FromValue};

use super::{
    Config, ConfigId, ConfigRepo, ConfigTag,
    HomoConfigContainer, CONFIG_RESERVED_FIELD_TAG, CONFIG_RESERVED_FIELD_ID,
};

type SomeConfigLoadFn = fn(
    &mut dyn Any,
    id: String,
    serde_json::Map<String, serde_json::Value>,
) -> Result<(), ConfigLoadError>;

pub struct ConfigRepositoryBuilder {
    repository: HashMap<TypeId, Box<dyn Any>>,
    tagged_loaders: HashMap<ConfigTag, (TypeId, SomeConfigLoadFn)>,
}

// todo: assert(typeof(parse_adding_to_any_store) == SomeConfigLoadFn)

pub enum ConfigRegistrationError {
    TypeAlreadyRegistered,
    TagAlreadyRegistered,
}

pub enum ConfigLoadError {
    ParseFailed(json::ParseError),
    StoreTypeInvalid,
    ConfigIdentifierOccupied,
    TagNotRegistered,
    NoCorrespondingStore,
}

fn parse_adding_to_any_store<C: Config>(
    dst: &mut dyn Any,
    id: String,
    src: serde_json::Map<String, serde_json::Value>,
) -> Result<(), ConfigLoadError> {
    dst.downcast_mut::<HomoConfigContainer<C>>()
        .ok_or(ConfigLoadError::StoreTypeInvalid)
        .and_then(|store| {
            let config = C::parse(src).map_err(ConfigLoadError::ParseFailed)?;
            match store.try_insert(id, config) {
                Ok(_) => Ok(()),
                Err(_) => Err(ConfigLoadError::ConfigIdentifierOccupied),
            }
        })
}

impl ConfigRepositoryBuilder {
    pub fn register<C: Config>(&mut self) -> Result<(), ConfigRegistrationError> {
        self.repository
            .try_insert(TypeId::of::<C>(), Box::new(HomoConfigContainer::<C>::new()))
            .map_err(|_| ConfigRegistrationError::TypeAlreadyRegistered)?;
        match self
            .tagged_loaders
            .try_insert(C::TAG, (TypeId::of::<C>(), parse_adding_to_any_store::<C>))
        {
            Ok(_) => Ok(()),
            Err(_) => Err(ConfigRegistrationError::TagAlreadyRegistered),
        }
    }

    pub fn build(self) -> ConfigRepo {
        ConfigRepo(self.repository)
    }

    pub fn add<C: Config>(&mut self, id: ConfigId<C>, config: C) -> Result<(), ()> {
        self.repository
            .get_mut(&TypeId::of::<C>())
            .ok_or(())
            .and_then(|store| {
                store
                    .downcast_mut::<HomoConfigContainer<C>>()
                    .ok_or(())
                    .and_then(|store| match store.try_insert(id.into_string(), config) {
                        Ok(_) => Ok(()),
                        Err(_) => Err(()),
                    })
            })
    }

    pub fn load(&mut self, src: serde_json::Value) -> Result<(), ConfigLoadError> {
        let mut src = json::Object::from_value(src).map_err(ConfigLoadError::ParseFailed)?;
        let tag = json::try_take_key::<String>(&mut src, CONFIG_RESERVED_FIELD_TAG).map_err(ConfigLoadError::ParseFailed)?;
        let id = json::try_take_key(&mut src, CONFIG_RESERVED_FIELD_ID).map_err(ConfigLoadError::ParseFailed)?;

        let (type_id, loader) = self.tagged_loaders.get(tag.as_str()).ok_or(ConfigLoadError::TagNotRegistered)?;
        let store = self.repository.get_mut(type_id).ok_or(ConfigLoadError::NoCorrespondingStore)?;
        loader(store, id, src)
    }
}
