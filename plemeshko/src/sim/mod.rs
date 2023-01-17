pub mod config;
pub mod erection;
pub mod units;

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::env::{config::Serializable, SimEnv};

use self::{
    config::resource::{RawResourceMap, ResourceMap},
    erection::{Erection, ErectionSnapshot, RawErectionSnapshot},
};

#[derive(Serialize, Deserialize)]
pub struct RawSimSnapshot {
    depot: RawResourceMap,
    erections: Vec<RawErectionSnapshot>,
}

pub struct SimSnapshot {
    depot: ResourceMap,
    erections: Vec<ErectionSnapshot>,
}

pub struct Sim {
    pub depot: ResourceMap,
    pub erections: Vec<Erection>,
    exited: bool,
}

impl Sim {
    pub const TICK_DELAY: Duration = Duration::from_secs(1);
    pub const TICK_THRESHOLD: Duration = Duration::from_millis(1);

    pub fn restore(env: &SimEnv, snapshot: SimSnapshot) -> anyhow::Result<Self> {
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

    pub fn step(&mut self, env: &SimEnv) -> anyhow::Result<()> {
        if self.exited {
            panic!("Sim is in exiting state when step was called");
        }
        for i in 0..self.erections.len() {
            self.erections[i].step(env, &mut self.depot)?;
        }
        Ok(())
    }
}

impl Serializable for SimSnapshot {
    type Raw = RawSimSnapshot;

    fn from_serializable(raw: Self::Raw, indexer: &mut crate::env::config::ConfigIndexer) -> Self {
        SimSnapshot {
            depot: Serializable::from_serializable(raw.depot, indexer),
            erections: Serializable::from_serializable(raw.erections, indexer),
        }
    }

    fn into_serializable(
        self,
        indexer: &mut crate::env::config::ConfigIndexer,
    ) -> anyhow::Result<Self::Raw> {
        Ok(RawSimSnapshot {
            depot: self.depot.into_serializable(indexer)?,
            erections: self.erections.into_serializable(indexer)?,
        })
    }
}
