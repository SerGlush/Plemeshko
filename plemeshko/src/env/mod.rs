pub mod text;

use std::{borrow::Cow, path::PathBuf};

use fluent::FluentArgs;
use plegine::config::{ConfigRepository, ConfigRepositoryBuilder};

use crate::sim::config;

use self::text::{TextIdentifier, TextRepository};

const CONFIG_DIR: &str = "config";
const TEXT_DIR: &str = "text";

pub struct SharedEnv {
    pub configs: ConfigRepository,
}

static_assertions::assert_impl_all!(SharedEnv: Sync);

pub struct AppEnv {
    pub shared: &'static SharedEnv,
    pub texts: TextRepository,
}

pub type SimEnv = SharedEnv;

impl SharedEnv {
    pub fn new() -> anyhow::Result<Self> {
        let mut config_repo_builder = ConfigRepositoryBuilder::new();
        config_repo_builder.register::<config::resource::Resource>()?;
        config_repo_builder.register::<config::setting_group::SettingGroup>()?;
        config_repo_builder.register::<config::transport::Transport>()?;
        config_repo_builder.register::<config::transport_group::TransportGroup>()?;
        config_repo_builder.register::<config::method::Method>()?;
        config_repo_builder.register::<config::method_group::MethodGroup>()?;
        let config_dir_path = PathBuf::from(CONFIG_DIR);
        config_repo_builder.load_directory(config_dir_path.as_path())?;
        Ok(SharedEnv {
            configs: config_repo_builder.build(),
        })
    }
}

impl AppEnv {
    pub fn new(shared: &'static SharedEnv) -> anyhow::Result<Self> {
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

    pub fn text<'a>(&'a self, id: &(impl TextIdentifier + ?Sized)) -> anyhow::Result<Cow<'a, str>> {
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
