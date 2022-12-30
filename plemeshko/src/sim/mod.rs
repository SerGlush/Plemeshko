pub mod config;
pub mod depot;
pub mod erection;
pub mod error;
pub mod transport_group;

use std::{io, path::PathBuf, time::Duration};

use depot::Depot;
use either::Either;
use erection::ErectionContainer;
use plegine::config::{ConfigLoadError, ConfigRepository, ConfigRepositoryBuilder};

use self::{config::transport::TransportId, erection::Erection, transport_group::TransportGroup};

pub struct Sim {
    depot: Depot,
    erections: ErectionContainer,
    configs: ConfigRepository,
}

const CONFIG_DIR: &'static str = "config";

impl Sim {
    pub const TICK_DELAY: Duration = Duration::from_secs(1);

    pub fn init() -> Result<Self, Either<ConfigLoadError, io::Error>> {
        let config_repo_builder = ConfigRepositoryBuilder::new();
        let config_dir_path = PathBuf::from(CONFIG_DIR);
        config_repo_builder.load_directory(config_dir_path.as_path())?;
        Ok(Sim {
            depot: Depot::new(),
            erections: ErectionContainer::new(),
            configs: config_repo_builder.build(),
        })
    }

    pub fn depot(&self) -> &Depot {
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
