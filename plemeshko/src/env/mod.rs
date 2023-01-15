use std::path::PathBuf;

use plegine::config::{ConfigRepository, ConfigRepositoryBuilder};

use crate::sim::config;

pub struct Env {
    pub configs: ConfigRepository,
}

const CONFIG_DIR: &str = "config";

impl Env {
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
        Ok(Env {
            configs: config_repo_builder.build(),
        })
    }
}
