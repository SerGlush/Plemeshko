use std::{any::TypeId, collections::HashMap, error::Error, fmt::Display, fs::File, io, path};

use either::Either;

use crate::json::{self, FromValue};

use super::{
    AnyHomoConfigContainer, Config, ConfigId, ConfigRepository, ConfigTag, HomoConfigContainer,
    CONFIG_RESERVED_FIELD_ID, CONFIG_RESERVED_FIELD_TAG,
};

type SomeConfigLoadFn = fn(
    &mut dyn AnyHomoConfigContainer,
    id: String,
    serde_json::Map<String, serde_json::Value>,
) -> Result<(), ConfigLoadError>;

pub struct ConfigRepositoryBuilder {
    repository: HashMap<TypeId, Box<dyn AnyHomoConfigContainer>>,
    tagged_loaders: HashMap<ConfigTag, (TypeId, SomeConfigLoadFn)>,
}

#[derive(Debug)]
pub enum ConfigRegistrationError {
    TypeAlreadyRegistered,
    TagAlreadyRegistered,
}

impl Display for ConfigRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigRegistrationError::TypeAlreadyRegistered => write!(f, "Type already registered"),
            ConfigRegistrationError::TagAlreadyRegistered => write!(f, "Tag already registered"),
        }
    }
}

impl Error for ConfigRegistrationError {}

#[derive(Debug)]
pub enum ConfigLoadError {
    ValueParseFailed(json::ParseError),
    JsonParseFailed(serde_json::Error),
    StoreTypeInvalid,
    ConfigIdentifierOccupied,
    TagNotRegistered(String),
    NoCorrespondingStore,
}

impl Display for ConfigLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigLoadError::ValueParseFailed(e) => write!(f, "Json value parsing failed: {e}"),
            ConfigLoadError::JsonParseFailed(e) => write!(f, "Json deserialization failed: {e}"),
            ConfigLoadError::StoreTypeInvalid => write!(
                f,
                "Attempted to downcast badly typed configuration storage."
            ),
            ConfigLoadError::ConfigIdentifierOccupied => {
                write!(f, "Attempred to load config with an already existing tag.")
            }
            ConfigLoadError::TagNotRegistered(tag) => write!(f, "Tag not registered: {tag}"),
            ConfigLoadError::NoCorrespondingStore => write!(
                f,
                "No configuration storage corresponding to the tagged TypeId found"
            ),
        }
    }
}

impl Error for ConfigLoadError {}

fn parse_adding_to_any_store<C: Config>(
    dst: &mut dyn AnyHomoConfigContainer,
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
            repository: HashMap::new(),
            tagged_loaders: HashMap::new(),
        }
    }

    pub fn register<C: Config>(&mut self) -> Result<(), ConfigRegistrationError> {
        let type_id = TypeId::of::<C>();
        self.repository
            .try_insert(type_id, Box::new(HomoConfigContainer::<C>::new()))
            .map_err(|_| ConfigRegistrationError::TypeAlreadyRegistered)?;
        match self
            .tagged_loaders
            .try_insert(C::TAG, (type_id, parse_adding_to_any_store::<C>))
        {
            Ok(_) => Ok(()),
            Err(_) => Err(ConfigRegistrationError::TagAlreadyRegistered),
        }
    }

    pub fn build(self) -> ConfigRepository {
        ConfigRepository(self.repository)
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
        let mut src = json::Object::from_value(src).map_err(ConfigLoadError::ValueParseFailed)?;
        let tag = json::try_take_key::<String>(&mut src, CONFIG_RESERVED_FIELD_TAG)
            .map_err(ConfigLoadError::ValueParseFailed)?;
        let id = json::try_take_key(&mut src, CONFIG_RESERVED_FIELD_ID)
            .map_err(ConfigLoadError::ValueParseFailed)?;

        let (type_id, loader) = self
            .tagged_loaders
            .get(tag.as_str())
            .ok_or(ConfigLoadError::TagNotRegistered(tag))?;
        let store = self
            .repository
            .get_mut(type_id)
            .ok_or(ConfigLoadError::NoCorrespondingStore)?;
        loader(store.as_mut(), id, src)
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
