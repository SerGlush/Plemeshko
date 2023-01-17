#[macro_use]
pub mod config;
pub mod text;

use std::{borrow::Cow, path::PathBuf};

use anyhow::Result;
use fluent::FluentArgs;

use crate::sim::config::resource::Resource;

use self::{
    config::{ConfigId, ConfigRepository, ConfigRepositoryBuilder},
    text::{TextIdentifier, TextRepository},
};

const CONFIG_DIR: &str = "config";
const TEXT_DIR: &str = "text";

const RESOURCE_LABEL_HUMAN: &str = "human";

pub struct SharedEnv {
    pub configs: ConfigRepository,
    pub human_id: ConfigId<Resource>,
}

static_assertions::assert_impl_all!(SharedEnv: Sync);

pub struct AppEnv {
    pub shared: &'static SharedEnv,
    pub texts: TextRepository,
}

pub type SimEnv = SharedEnv;

impl SharedEnv {
    pub fn new() -> Result<Self> {
        let mut configs_builder = ConfigRepositoryBuilder::new();
        crate::sim::config::register(&mut configs_builder)?;
        configs_builder.load_directory(&PathBuf::from(CONFIG_DIR))?;
        let configs = configs_builder.build()?;
        let human_id = configs.indexer.get_id(RESOURCE_LABEL_HUMAN.to_owned())?;
        Ok(SharedEnv { configs, human_id })
    }
}

impl AppEnv {
    pub fn new(shared: &'static SharedEnv) -> Result<Self> {
        Ok(AppEnv {
            shared,
            texts: TextRepository::new()?,
        })
    }

    pub fn configs(&self) -> &'static ConfigRepository {
        &self.shared.configs
    }

    fn text_impl<'a>(
        &'a self,
        id: &(impl TextIdentifier + ?Sized),
        args: Option<&'a FluentArgs<'_>>,
    ) -> anyhow::Result<Cow<'a, str>> {
        self.texts.get(id, args).or_else(|e| match e {
            text::TextRetrievalError::NotFound(id) => Ok(Cow::Owned(id)),
            e => Err(e.into()),
        })
    }

    pub fn text<'a>(&'a self, id: &(impl TextIdentifier + ?Sized)) -> Result<Cow<'a, str>> {
        self.text_impl(id, None)
    }

    pub fn text_fmt<'a>(
        &'a self,
        id: &(impl TextIdentifier + ?Sized),
        args: &'a FluentArgs<'_>,
    ) -> anyhow::Result<Cow<'a, str>> {
        self.text_impl(id, Some(args))
    }
}
