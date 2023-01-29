pub mod config;
pub mod erection;
pub mod units;

use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::state::{
    components::{ComponentsRef, SharedComponents},
    serializable::Serializable,
    SharedState,
};

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

    pub fn restore(shared_comps: &SharedComponents, snapshot: SimSnapshot) -> anyhow::Result<Self> {
        let SimSnapshot { depot, erections } = snapshot;
        Ok(Sim {
            depot,
            erections: erections
                .into_iter()
                .map(|s| Erection::restore(shared_comps, s))
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

    pub fn step(&mut self, env: &SharedState) -> anyhow::Result<()> {
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

    fn from_serializable(raw: Self::Raw, ctx: ComponentsRef<'_>) -> Result<Self> {
        Ok(SimSnapshot {
            depot: Serializable::from_serializable(raw.depot, ctx)?,
            erections: Serializable::from_serializable(raw.erections, ctx)?,
        })
    }

    fn into_serializable(self, ctx: ComponentsRef<'_>) -> Result<Self::Raw> {
        Ok(RawSimSnapshot {
            depot: self.depot.into_serializable(ctx)?,
            erections: self.erections.into_serializable(ctx)?,
        })
    }
}
