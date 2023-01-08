pub mod config;
pub mod erection;
pub mod error;
pub mod transport_group;
pub mod units;

use std::{error::Error, path::PathBuf, time::Duration};

use erection::ErectionContainer;
use plegine::config::{ConfigRepository, ConfigRepositoryBuilder};

use self::{
    config::{resource::storage::ResourceStorage, transport::TransportId},
    erection::Erection,
    error::SimResult,
    transport_group::TransportGroup,
};

pub struct Sim {
    depot: ResourceStorage,
    erections: ErectionContainer,
    configs: ConfigRepository,
    exited: bool,
}

static_assertions::assert_not_impl_all!(Sim: Drop);

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
            exited: false,
        })
    }

    pub fn depot(&self) -> &ResourceStorage {
        &self.depot
    }

    pub fn erections(&self) -> impl Iterator<Item = &Erection> {
        self.erections.iter()
    }

    pub fn exited(&self) -> bool {
        self.exited
    }

    pub fn exit(&mut self) {
        self.exited = true;
        // todo: finalization code, mb dropping resources / saving state
    }

    pub fn default_transport(&self, tg: TransportGroup) -> TransportId {
        match tg {
            TransportGroup::Gas => todo!(),
            TransportGroup::Liquid => todo!(),
            TransportGroup::Solid => todo!(),
        }
    }

    pub fn step(&mut self) -> SimResult<()> {
        if self.exited {
            panic!("Sim is in exiting state when step was called");
        }
        for i in 0..self.erections.len() {
            self.erections[i].step(&mut self.depot, &self.configs)?;
        }
        Ok(())
    }
}
