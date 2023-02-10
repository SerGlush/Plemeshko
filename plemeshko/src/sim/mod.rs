pub mod config;
pub mod production;
pub mod units;

use std::{time::Duration, ops::Deref};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::state::{
    components::{ComponentsRef, SharedComponents},
    serializable::Serializable,
    SharedState,
};

use self::{
    config::resource::{RawResourceMap, ResourceMap},
    production::{Production, ProductionSnapshot, RawProductionSnapshot},
};

#[derive(Serialize, Deserialize)]
pub struct RawSimSnapshot {
    depot: RawResourceMap,
    productions: Vec<RawProductionSnapshot>,
    nutrition: i32,
}

pub struct SimSnapshot {
    depot: ResourceMap,
    productions: Vec<ProductionSnapshot>,

    nutrition: i32,
}

pub struct Sim {
    pub depot: ResourceMap,
    pub productions: Vec<Production>,
    exited: bool,

    pub nutrition: i32,
}

impl Sim {
    pub const TICK_DELAY: Duration = Duration::from_secs(1);
    pub const TICK_THRESHOLD: Duration = Duration::from_millis(1);

    pub fn restore(shared_comps: &SharedComponents, snapshot: SimSnapshot) -> anyhow::Result<Self> {
        let SimSnapshot { depot, productions, nutrition } = snapshot;
        Ok(Sim {
            depot,
            productions: productions
                .into_iter()
                .map(|s| Production::restore(shared_comps, s))
                .try_collect()?,
            exited: false,
            nutrition,
        })
    }

    pub fn snapshot(&self) -> SimSnapshot {
        SimSnapshot {
            depot: self.depot.clone(),
            productions: self.productions.iter().map(Production::snapshot).collect(),
            nutrition: self.nutrition.clone(),
        }
    }

    pub fn new() -> Self {
        Sim {
            depot: ResourceMap::new(),
            productions: Vec::new(),
            exited: false,
            nutrition: 100,
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

        let population = self.depot.get(&env.human_id).copied().unwrap_or_default();

        for i in 0..self.productions.len() {
            self.productions[i].step(env, &mut self.depot)?;
        }
        Ok(())
    }
}

impl Serializable for SimSnapshot {
    type Raw = RawSimSnapshot;

    fn from_serializable(raw: Self::Raw, ctx: ComponentsRef<'_>) -> Result<Self> {
        Ok(SimSnapshot {
            depot: Serializable::from_serializable(raw.depot, ctx)?,
            productions: Serializable::from_serializable(raw.productions, ctx)?,
            nutrition: raw.nutrition,
        })
    }

    fn into_serializable(self, ctx: ComponentsRef<'_>) -> Result<Self::Raw> {
        Ok(RawSimSnapshot {
            depot: self.depot.into_serializable(ctx)?,
            productions: self.productions.into_serializable(ctx)?,
            nutrition: self.nutrition,
        })
    }
}
