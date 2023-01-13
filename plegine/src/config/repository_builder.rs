use std::{
    any::TypeId,
    collections::HashMap,
    error::Error,
    fmt::Display,
    fs::File,
    io,
    path::{self, Path, PathBuf},
};

use either::Either;
use serde::Deserialize;
use serde_json::value::RawValue;

use super::{AnyHomoConfigContainer, Config, ConfigRepository, HomoConfigContainer};

#[derive(Deserialize)]
pub struct RawConfig {
    pub tag: String,
    pub name: String,
    // todo: something similar to `#[serde(flatten)]` (or custom deserializer?) can remove unnecessary nesting
    pub payload: Box<RawValue>,
}

type SomeConfigLoadFn = fn(
    &mut dyn AnyHomoConfigContainer,
    &Path,
    id: String,
    raw_cfg: &RawValue,
) -> Result<(), ConfigLoadError>;

pub struct ConfigRepositoryBuilder {
    repository: HashMap<TypeId, Box<dyn AnyHomoConfigContainer>>,
    tagged_loaders: HashMap<&'static str, (TypeId, SomeConfigLoadFn)>,
}

#[derive(Debug)]
pub enum ConfigRegistrationError {
    TypeAlreadyRegistered,
    TagAlreadyRegistered,
}

#[derive(Debug)]
pub enum ConfigLoadErrorPayload {
    ConfigIdentifierOccupied,
    NoCorrespondingStore,
    ParsingFailed(serde_json::Error),
    StoreTypeInvalid,
    TagNotRegistered(String),
}

use ConfigLoadErrorPayload::*;

impl ConfigLoadError {
    pub fn new(path: &Path, payload: ConfigLoadErrorPayload) -> Self {
        ConfigLoadError {
            path: path.to_owned(),
            payload,
        }
    }
}

#[derive(Debug)]
pub struct ConfigLoadError {
    pub path: PathBuf,
    pub payload: ConfigLoadErrorPayload,
}

fn parse_adding_to_any_store<C: Config>(
    dst: &mut dyn AnyHomoConfigContainer,
    path: &Path,
    id: String,
    raw_cfg: &RawValue,
) -> Result<(), ConfigLoadError> {
    dst.downcast_mut::<HomoConfigContainer<C>>()
        .ok_or(ConfigLoadError::new(path, StoreTypeInvalid))
        .and_then(|store| {
            let config = serde_json::from_str(raw_cfg.get())
                .map_err(|e| ConfigLoadError::new(path, ParsingFailed(e)))?;
            match store.try_insert(id, config) {
                Ok(_) => Ok(()),
                Err(_) => Err(ConfigLoadError::new(path, ConfigIdentifierOccupied)),
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

    pub fn load_raw(&mut self, path: &Path, raw: RawConfig) -> Result<(), ConfigLoadError> {
        let (type_id, loader) =
            self.tagged_loaders
                .get(raw.tag.as_str())
                .ok_or_else(|| ConfigLoadError {
                    path: path.to_owned(),
                    payload: TagNotRegistered(raw.tag),
                })?;
        let store = self
            .repository
            .get_mut(type_id)
            .ok_or(ConfigLoadError::new(path, NoCorrespondingStore))?;
        loader(store.as_mut(), path, raw.name, raw.payload.as_ref())
    }

    pub fn load_file(
        &mut self,
        path: &path::Path,
    ) -> Result<(), Either<ConfigLoadError, io::Error>> {
        let file = File::open(path).map_err(Either::Right)?;
        let reader = io::BufReader::new(file);
        let raw_cfgs = serde_json::from_reader::<_, Vec<RawConfig>>(reader)
            .map_err(|e| Either::Left(ConfigLoadError::new(path, ParsingFailed(e))))?;
        for raw_cfg in raw_cfgs {
            self.load_raw(path, raw_cfg).map_err(Either::Left)?;
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

impl Display for ConfigRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigRegistrationError::TypeAlreadyRegistered => write!(f, "Type already registered"),
            ConfigRegistrationError::TagAlreadyRegistered => write!(f, "Tag already registered"),
        }
    }
}

impl Error for ConfigRegistrationError {}

impl Display for ConfigLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = self.path.display();
        write!(f, "When loading config file \"{path}\": ")?;
        match &self.payload {
            ParsingFailed(e) => write!(f, "Parsing failed: {e}"),
            StoreTypeInvalid => write!(
                f,
                "Attempted to downcast badly typed configuration storage."
            ),
            ConfigIdentifierOccupied => {
                write!(f, "Attempred to load config with an already existing tag.")
            }
            TagNotRegistered(tag) => write!(f, "Tag not registered: {tag}"),
            NoCorrespondingStore => write!(
                f,
                "No configuration storage corresponding to the tagged TypeId found"
            ),
        }
    }
}

impl Error for ConfigLoadError {}
