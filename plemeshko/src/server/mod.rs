pub mod config;
pub mod erection;
pub mod error;
pub mod units;

use std::{error::Error, path::PathBuf, time::Duration};

use plegine::config::{ConfigRepository, ConfigRepositoryBuilder};

use self::{config::resource::storage::ResourceMap, erection::Erection, error::SimResult};

pub struct Sim {
    pub depot: ResourceMap,
    pub erections: Vec<Erection>,
    pub configs: ConfigRepository,
    exited: bool,
}

static_assertions::assert_not_impl_all!(Sim: Drop);

const CONFIG_DIR: &'static str = "config";

impl Sim {
    pub const TICK_DELAY: Duration = Duration::from_secs(1);
    pub const TICK_THRESHOLD: Duration = Duration::from_millis(1);

    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut config_repo_builder = ConfigRepositoryBuilder::new();
        config_repo_builder
            .register::<config::resource::Resource>()
            .map_err(Box::new)?;
        config_repo_builder
            .register::<config::setting_group::SettingGroup>()
            .map_err(Box::new)?;
        config_repo_builder
            .register::<config::transport::Transport>()
            .map_err(Box::new)?;
        config_repo_builder
            .register::<config::transport_group::TransportGroup>()
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
            depot: ResourceMap::new(),
            erections: Vec::new(),
            configs: config_repo_builder.build(),
            exited: false,
        })
    }

    pub fn exited(&self) -> bool {
        self.exited
    }

    pub fn exit(&mut self) {
        self.exited = true;
        // todo: finalization code, mb dropping resources / saving state
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
