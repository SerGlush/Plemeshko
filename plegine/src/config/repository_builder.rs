use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    fmt::Display,
    fs::File,
    io,
    path::{self, Path, PathBuf},
};

use either::Either;
use serde::Deserialize;
use serde_json::value::RawValue;
use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum ConfigRegistrationError {
    #[error("Type already registered: {type_name}")]
    TypeAlreadyRegistered { type_name: &'static str },
    #[error("Tag already registered: {tag}")]
    TagAlreadyRegistered { tag: &'static str },
}

#[derive(Debug, Error)]
pub enum ConfigLoadErrorPayload {
    #[error("Identifier already loaded: {id}")]
    IdAlreadyOccupied { id: String },
    // #[error("Type not registered: {type_name}")]
    // TypeNotRegistered { type_name: &'static str },
    #[error("Storage for requested tag doesn't exist: {tag}")]
    StoreNotFound { tag: String },
    #[error("Parsing failed: {0}")]
    ParsingFailed(#[from] serde_json::Error),
    #[error("Storage had type not matching its key, expected: {type_name}")]
    StoreTypeInvalid { type_name: &'static str },
    #[error("Tag not registered: {tag}")]
    TagNotRegistered { tag: String },
}

use ConfigLoadErrorPayload::*;

#[derive(Debug, Error)]
pub struct ConfigLoadError {
    pub path: PathBuf,
    #[source]
    pub payload: ConfigLoadErrorPayload,
}

impl ConfigLoadError {
    fn new(path: &Path, payload: ConfigLoadErrorPayload) -> Self {
        ConfigLoadError {
            path: path.to_owned(),
            payload,
        }
    }
}

fn parse_adding_to_any_store<C: Config>(
    dst: &mut dyn AnyHomoConfigContainer,
    path: &Path,
    id: String,
    raw_cfg: &RawValue,
) -> Result<(), ConfigLoadError> {
    dst.downcast_mut::<HomoConfigContainer<C>>()
        .ok_or(ConfigLoadError::new(
            path,
            StoreTypeInvalid {
                type_name: type_name::<C>(),
            },
        ))
        .and_then(|store| {
            let config = serde_json::from_str(raw_cfg.get())
                .map_err(|e| ConfigLoadError::new(path, ParsingFailed(e)))?;
            match store.try_insert(id, config) {
                Ok(_) => Ok(()),
                Err(e) => Err(ConfigLoadError::new(
                    path,
                    IdAlreadyOccupied {
                        id: e.entry.key().clone(),
                    },
                )),
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
            .map_err(|_| ConfigRegistrationError::TypeAlreadyRegistered {
                type_name: type_name::<C>(),
            })?;
        match self
            .tagged_loaders
            .try_insert(C::TAG, (type_id, parse_adding_to_any_store::<C>))
        {
            Ok(_) => Ok(()),
            Err(_) => Err(ConfigRegistrationError::TagAlreadyRegistered { tag: C::TAG }),
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
                    payload: TagNotRegistered {
                        tag: raw.tag.clone(),
                    },
                })?;
        let store = self
            .repository
            .get_mut(type_id)
            .ok_or(ConfigLoadError::new(path, StoreNotFound { tag: raw.tag }))?;
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

impl Display for ConfigLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "When loading config file \"{}\": ", self.path.display())?;
        self.payload.fmt(f)
    }
}
