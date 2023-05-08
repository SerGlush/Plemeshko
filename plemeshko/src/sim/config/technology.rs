use anyhow::Result;
use serde::Deserialize;

use crate::state::{
    config::{Config, FatConfigId, FatConfigLabel, Info, Prepare, RawInfo},
    research::Research,
};

use super::{
    production_method::{ProductionMethod, ProductionMethodId},
    transport_method::{TransportMethod, TransportMethodId},
};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawTechnologyBonus {
    UnlockTransport(FatConfigLabel<TransportMethod>),
    UnlockProduction(FatConfigLabel<ProductionMethod>),
}

#[derive(Clone, Copy, Debug)]
pub enum TechnologyBonus {
    UnlockTransport(TransportMethodId),
    UnlockProduction(ProductionMethodId),
}

#[derive(Deserialize)]
pub struct RawTechnology {
    #[serde(flatten)]
    pub info: RawInfo,
    pub bonuses: Vec<RawTechnologyBonus>,
    pub prerequisites: Vec<FatConfigLabel<Technology>>,
    pub cost: u64,
}

#[derive(Debug)]
pub struct Technology {
    pub info: Info,
    pub bonuses: Vec<TechnologyBonus>,
    pub prerequisites: Vec<TechnologyId>,
    pub cost: u64,

    prerequisites_satisfied: bool,
}

pub type TechnologyId = FatConfigId<Technology>;

impl Technology {
    pub fn check_prerequisites(&mut self, research: &Research) {
        self.prerequisites_satisfied = self
            .prerequisites
            .iter()
            .all(|id| research.is_researched(*id));
    }

    pub fn prerequisites_satisfied(&self) -> bool {
        self.prerequisites_satisfied
    }
}

impl Prepare for RawTechnologyBonus {
    type Prepared = TechnologyBonus;

    fn prepare(
        self,
        ctx: &mut crate::state::config::ConfigsLoadingContext<'_>,
        tif: &mut crate::state::text::TextIdFactory,
    ) -> Result<Self::Prepared> {
        tif.with_lock(|tif| {
            Ok(match self {
                RawTechnologyBonus::UnlockTransport(id) => {
                    TechnologyBonus::UnlockTransport(id.prepare(ctx, tif)?)
                }
                RawTechnologyBonus::UnlockProduction(id) => {
                    TechnologyBonus::UnlockProduction(id.prepare(ctx, tif)?)
                }
            })
        })
    }
}

impl Prepare for RawTechnology {
    type Prepared = Technology;

    fn prepare(
        self,
        ctx: &mut crate::state::config::ConfigsLoadingContext<'_>,
        tif: &mut crate::state::text::TextIdFactory,
    ) -> Result<Self::Prepared> {
        let info = self.info.prepare(ctx, tif)?;
        tif.with_lock(|tif| {
            Ok(Technology {
                info,
                bonuses: self.bonuses.prepare(ctx, tif)?,
                prerequisites: self.prerequisites.prepare(ctx, tif)?,
                cost: self.cost,
                prerequisites_satisfied: false,
            })
        })
    }
}

impl Config for Technology {
    type Raw = RawTechnology;

    const TAG: &'static str = "technology";
}
