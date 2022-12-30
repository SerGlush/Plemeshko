use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fs::File,
    io, path,
};

use either::Either;

use crate::json::{self, FromValue};

use super::{
    Config, ConfigId, ConfigRepository, ConfigTag, HomoConfigContainer, CONFIG_RESERVED_FIELD_ID,
    CONFIG_RESERVED_FIELD_TAG,
};

type SomeConfigLoadFn = fn(
    &mut dyn Any,
    id: String,
    serde_json::Map<String, serde_json::Value>,
) -> Result<(), ConfigLoadError>;

pub struct ConfigRepositoryBuilder {
    repo: HashMap<TypeId, Box<dyn Any>>,
    tagged_loaders: HashMap<ConfigTag, (TypeId, SomeConfigLoadFn)>,
}

pub enum ConfigRegistrationError {
    TypeAlreadyRegistered,
    TagAlreadyRegistered,
}

pub enum ConfigLoadError {
    ValueParseFailed(json::ParseError),
    JsonParseFailed(serde_json::Error),
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
            let config = C::parse(src).map_err(ConfigLoadError::ValueParseFailed)?;
            match store.try_insert(id, config) {
                Ok(_) => Ok(()),
                Err(_) => Err(ConfigLoadError::ConfigIdentifierOccupied),
            }
        })
}

impl ConfigRepositoryBuilder {
    pub fn new() -> Self {
        ConfigRepositoryBuilder {
            repo: HashMap::new(),
            tagged_loaders: HashMap::new(),
        }
    }

    pub fn register<C: Config>(&mut self) -> Result<(), ConfigRegistrationError> {
        self.repo
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

    pub fn build(self) -> ConfigRepository {
        ConfigRepository(self.repo)
    }

    pub fn add<C: Config>(&mut self, id: ConfigId<C>, config: C) -> Result<(), ()> {
        self.repo
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
        let mut src = json::Object::from_value(src).map_err(ConfigLoadError::ValueParseFailed)?;
        let tag = json::try_take_key::<String>(&mut src, CONFIG_RESERVED_FIELD_TAG)
            .map_err(ConfigLoadError::ValueParseFailed)?;
        let id = json::try_take_key(&mut src, CONFIG_RESERVED_FIELD_ID)
            .map_err(ConfigLoadError::ValueParseFailed)?;

        let (type_id, loader) = self
            .tagged_loaders
            .get(tag.as_str())
            .ok_or(ConfigLoadError::TagNotRegistered)?;
        let store = self
            .repo
            .get_mut(type_id)
            .ok_or(ConfigLoadError::NoCorrespondingStore)?;
        loader(store, id, src)
    }

    pub fn load_file(
        &mut self,
        path: &path::Path,
    ) -> Result<(), Either<ConfigLoadError, io::Error>> {
        let file = File::open(path).map_err(Either::Right)?;
        let reader = io::BufReader::new(file);
        let value = serde_json::from_reader::<_, json::Value>(reader)
            .map_err(|e| Either::Left(ConfigLoadError::JsonParseFailed(e)))?;
        let array = json::Array::from_value(value)
            .map_err(|e| Either::Left(ConfigLoadError::ValueParseFailed(e)))?;
        for element in array {
            self.load(element).map_err(Either::Left)?;
        }
        Ok(())
    }

    pub fn load_directory(
        &mut self,
        path: &path::Path,
    ) -> Result<(), Either<ConfigLoadError, io::Error>> {
        for dir_entry in std::fs::read_dir(path).map_err(Either::Right)? {
            let dir_entry = dir_entry.map_err(Either::Right)?;
            let entry_path = dir_entry.path();
            if entry_path.is_file() {
                self.load_file(&entry_path)?;
            } else if entry_path.is_dir() {
                self.load_directory(&entry_path)?;
            }
        }
        Ok(())
    }
}
