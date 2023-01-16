pub mod config;
pub mod erection;
pub mod units;

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::env::Env;

use self::{
    config::resource::storage::ResourceMap,
    erection::{Erection, ErectionSnapshot},
};

#[derive(Serialize, Deserialize)]
pub struct SimSnapshot {
    depot: ResourceMap,
    erections: Vec<ErectionSnapshot>,
}

pub struct Sim {
    pub depot: ResourceMap,
    pub erections: Vec<Erection>,
    exited: bool,
}

pub const RESOURCE_ID_HUMAN: &str = "human";

impl Sim {
    pub const TICK_DELAY: Duration = Duration::from_secs(1);
    pub const TICK_THRESHOLD: Duration = Duration::from_millis(1);

    pub fn restore(env: &Env, snapshot: SimSnapshot) -> anyhow::Result<Self> {
        let SimSnapshot { depot, erections } = snapshot;
        Ok(Sim {
            depot,
            erections: erections
                .into_iter()
                .map(|s| Erection::restore(env, s))
                .try_collect()?,
            exited: false,
        })
    }

    pub fn snapshot(&self) -> SimSnapshot {
        SimSnapshot {
            depot: self.depot.clone(),
            erections: self.erections.iter().map(Erection::snapshot).collect(),
        }
    }

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

    pub fn step(&mut self, env: &Env) -> anyhow::Result<()> {
        if self.exited {
            panic!("Sim is in exiting state when step was called");
        }
        for i in 0..self.erections.len() {
            self.erections[i].step(env, &mut self.depot)?;
        }
        Ok(())
    }
}
