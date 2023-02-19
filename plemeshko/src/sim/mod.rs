pub mod config;
pub mod production;
pub mod units;

use std::{cmp::Ordering, time::Duration};

use anyhow::Result;
use rand::random;
use rodio::Source;
use serde::{Deserialize, Serialize};

use crate::{
    state::{
        components::{ComponentsRef, SharedComponents},
        serializable::Serializable,
        SharedState,
    },
    util::cor::Cor,
};

use self::{
    config::resource::{RawResourceMap, ResourceMap},
    production::{Production, ProductionSnapshot, RawProductionSnapshot},
    units::ResourceAmount,
};

#[derive(Serialize, Deserialize)]
pub struct RawSimSnapshot {
    depot: RawResourceMap,
    productions: Vec<RawProductionSnapshot>,

    nutrition: i64,
    pop_growth_stack: f64,
}

pub struct SimSnapshot {
    depot: ResourceMap,
    productions: Vec<ProductionSnapshot>,

    nutrition: i64,
    pop_growth_stack: f64,
}

pub struct Sim {
    pub depot: ResourceMap,
    pub productions: Vec<Production>,
    exited: bool,

    pub nutrition: i64,
    pub pop_growth_stack: f64,
}

impl Sim {
    pub const TICK_DELAY: Duration = Duration::from_secs(1);
    pub const TICK_THRESHOLD: Duration = Duration::from_millis(1);

    pub fn restore(shared_comps: &SharedComponents, snapshot: SimSnapshot) -> anyhow::Result<Self> {
        let SimSnapshot {
            depot,
            productions,
            nutrition,
            pop_growth_stack,
        } = snapshot;
        Ok(Sim {
            depot,
            productions: productions
                .into_iter()
                .map(|s| Production::restore(shared_comps, s))
                .try_collect()?,
            exited: false,
            nutrition,
            pop_growth_stack,
        })
    }

    pub fn snapshot(&self) -> SimSnapshot {
        SimSnapshot {
            depot: self.depot.clone(),
            productions: self.productions.iter().map(Production::snapshot).collect(),
            nutrition: self.nutrition,
            pop_growth_stack: self.pop_growth_stack,
        }
    }

    pub fn new() -> Self {
        Sim {
            depot: ResourceMap::new(),
            productions: Vec::new(),
            exited: false,
            nutrition: 100,
            pop_growth_stack: 0.0,
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
        env.play_sfx(
            rodio::source::SineWave::new(440.0)
                .take_duration(std::time::Duration::from_millis(300))
                .amplify(0.2),
        );

        if self.exited {
            panic!("Sim is in exiting state when step was called");
        }

        for i in 0..self.productions.len() {
            self.productions[i].step(env, &mut self.depot)?;
        }

        if self.nutrition > 10 {
            self.nutrition -= 10;
        } else {
            self.nutrition -= self.nutrition;
        }

        let population = self.depot.get(&env.human_id).copied().unwrap_or_default();
        let mut depot_food = self.depot.get(&env.food_id).copied().unwrap_or_default();
        let food_need_value = (100 - self.nutrition) * 8 / 10;
        let food_needed = food_need_value * population.0;
        let food_eaten = match depot_food.0.cmp(&food_needed) {
            Ordering::Less => {
                let nutr_eaten = depot_food.0;
                depot_food.0 = 0;
                nutr_eaten
            }
            Ordering::Equal => {
                depot_food.0 = 0;
                food_needed
            }
            Ordering::Greater => {
                depot_food.0 -= food_needed;
                food_needed
            }
        };

        let mut nutrition_increase = (10 * food_eaten) as f64 / (8.0 * population.0 as f64);
        self.nutrition += nutrition_increase.floor() as i64;
        nutrition_increase -= nutrition_increase.floor();
        if random::<f64>() < nutrition_increase {
            self.nutrition += 1;
        }

        self.pop_growth_stack += population.0 as f64 * ((self.nutrition - 50) as f64 / 10000.0);

        self.depot.cor_put(
            &env.human_id,
            ResourceAmount(self.pop_growth_stack.ceil() as i64),
        );
        self.depot
            .cor_put(&env.food_id, ResourceAmount(-food_eaten));

        self.pop_growth_stack -= self.pop_growth_stack.ceil();

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
            pop_growth_stack: raw.pop_growth_stack,
        })
    }

    fn into_serializable(self, ctx: ComponentsRef<'_>) -> Result<Self::Raw> {
        Ok(RawSimSnapshot {
            depot: self.depot.into_serializable(ctx)?,
            productions: self.productions.into_serializable(ctx)?,
            nutrition: self.nutrition,
            pop_growth_stack: self.pop_growth_stack,
        })
    }
}
