pub mod config;
pub mod erection;
pub mod error;
pub mod units;

use std::time::Duration;

use crate::env::Env;

use self::{config::resource::storage::ResourceMap, erection::Erection, error::SimResult};

pub struct Sim {
    pub depot: ResourceMap,
    pub erections: Vec<Erection>,
    exited: bool,
}

static_assertions::assert_not_impl_all!(Sim: Drop);

impl Sim {
    pub const TICK_DELAY: Duration = Duration::from_secs(1);
    pub const TICK_THRESHOLD: Duration = Duration::from_millis(1);

    pub fn new() -> Self {
        Sim {
            depot: ResourceMap::new(),
            erections: Vec::new(),
            exited: false,
        }
    }

    pub fn exited(&self) -> bool {
        self.exited
    }

    pub fn exit(&mut self) {
        self.exited = true;
        // todo: finalization code, mb dropping resources / saving state
    }

    pub fn step(&mut self, env: &Env) -> SimResult<()> {
        if self.exited {
            panic!("Sim is in exiting state when step was called");
        }
        for i in 0..self.erections.len() {
            self.erections[i].step(env, &mut self.depot)?;
        }
        Ok(())
    }
}
