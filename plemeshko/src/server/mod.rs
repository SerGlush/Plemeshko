pub mod config;
pub mod erection;
pub mod error;
pub mod transport_group;
pub mod units;

use std::{
    error::Error,
    path::PathBuf,
    time::Duration,
};

use erection::ErectionContainer;
use plegine::config::{ConfigRepository, ConfigRepositoryBuilder};

use self::{
    config::{resource::storage::ResourceStorage, transport::TransportId},
    erection::Erection,
    transport_group::TransportGroup,
};

pub struct Sim {
    depot: ResourceStorage,
    erections: ErectionContainer,
    configs: ConfigRepository,
}

const CONFIG_DIR: &'static str = "config";

impl Sim {
    pub const TICK_DELAY: Duration = Duration::from_secs(1);
    pub const TICK_THRESHOLD: Duration = Duration::from_millis(20);

    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut config_repo_builder = ConfigRepositoryBuilder::new();
        config_repo_builder
            .register::<config::resource::Resource>()
            .map_err(Box::new)?;
        config_repo_builder
            .register::<config::transport::Transport>()
            .map_err(Box::new)?;
        config_repo_builder
            .register::<config::method::Method>()
            .map_err(Box::new)?;
        config_repo_builder
            .register::<config::method_group::MethodGroup>()
            .map_err(Box::new)?;
        let config_dir_path = PathBuf::from(CONFIG_DIR);
        config_repo_builder.load_directory(config_dir_path.as_path())?;

        Ok(Sim {
            depot: ResourceStorage::new(),
            erections: ErectionContainer::new(),
            configs: config_repo_builder.build(),
        })
    }

    pub fn depot(&self) -> &ResourceStorage {
        &self.depot
    }

    pub fn erections(&self) -> impl Iterator<Item = &Erection> {
        self.erections.iter()
    }

    pub fn default_transport(&self, tg: TransportGroup) -> TransportId {
        match tg {
            TransportGroup::Gas => todo!(),
            TransportGroup::Liquid => todo!(),
            TransportGroup::Solid => todo!(),
        }
    }

    pub fn step(&mut self) {}
}
