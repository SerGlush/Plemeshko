pub mod config;
pub mod depot;
pub mod erection;
pub mod error;

use std::time::Duration;

use depot::Depot;
use erection::ErectionContainer;

use self::erection::Erection;

pub struct Sim {
    main_depot: Depot,
    erections: ErectionContainer,
}

impl Sim {
    pub const TICK_DELAY: Duration = Duration::from_secs(1);

    pub fn main_depot(&self) -> &Depot {
        &self.main_depot
    }

    pub fn erections(&self) -> impl Iterator<Item = &Erection> {
        self.erections.iter()
    }

    pub fn step(&mut self) {
        
    }
}

impl Default for Sim {
    fn default() -> Self {
        Sim {
            main_depot: Depot::new(),
            erections: ErectionContainer::new(),
        }
    }
}
