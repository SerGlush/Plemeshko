use std::{collections::HashSet, ops::AddAssign, sync::RwLock};

use serde::{Deserialize, Serialize};

use crate::sim::config::{
    production_method::{ProductionMethod, ProductionMethodId},
    technology::{Technology, TechnologyBonus, TechnologyId},
    transport_method::{TransportMethod, TransportMethodId},
};

use super::{components::SharedComponents, config::FatConfigLabel, serializable::Serializable};

#[derive(Serialize, Deserialize)]
pub struct RawResearch {
    current: Option<(FatConfigLabel<Technology>, u64)>,
    finished: HashSet<FatConfigLabel<Technology>>,
    unlocked_transport: HashSet<FatConfigLabel<TransportMethod>>,
    unlocked_production: HashSet<FatConfigLabel<ProductionMethod>>,
}

#[derive(Clone)]
pub struct Research {
    current: Option<(TechnologyId, u64)>,
    researched: HashSet<TechnologyId>,
    unlocked_transport: HashSet<TransportMethodId>,
    unlocked_production: HashSet<ProductionMethodId>,
}

impl Research {
    pub fn new(shared_comps: &mut SharedComponents) -> anyhow::Result<Self> {
        let mut unlocked_production = HashSet::new();
        for production_method in shared_comps.iter_configs::<ProductionMethod>() {
            let (production_method_id, production_method) = production_method?;
            if production_method.initially_unlocked {
                unlocked_production.insert(production_method_id);
            }
        }
        let mut unlocked_transport = HashSet::new();
        for transport_method in shared_comps.iter_configs::<TransportMethod>() {
            let (transport_method_id, transport_method) = transport_method?;
            if transport_method.initially_unlocked {
                unlocked_transport.insert(transport_method_id);
            }
        }
        let research = Research {
            current: None,
            researched: HashSet::new(),
            unlocked_transport,
            unlocked_production,
        };
        research.update_technology_satisfaction(shared_comps)?;
        Ok(research)
    }

    pub fn update_technology_satisfaction(
        &self,
        shared_comps: &mut SharedComponents,
    ) -> anyhow::Result<()> {
        for technology in shared_comps.iter_configs_mut::<Technology>() {
            technology?.1.check_prerequisites(self);
        }
        Ok(())
    }

    pub fn step(&mut self, shared_comps: &RwLock<SharedComponents>) -> anyhow::Result<()> {
        if let &mut Some((id, ref mut progress)) = &mut self.current {
            progress.add_assign(1);
            if *progress >= shared_comps.read().unwrap().config(id).unwrap().cost {
                self.researched.insert(id);
                let mut shared_comps = shared_comps.write().unwrap();
                for bonus in &shared_comps.config(id)?.bonuses {
                    // todo: log when double-unlock
                    match *bonus {
                        TechnologyBonus::UnlockTransport(tr_id) => {
                            self.unlocked_transport.insert(tr_id)
                        }
                        TechnologyBonus::UnlockProduction(pr_id) => {
                            self.unlocked_production.insert(pr_id)
                        }
                    };
                }
                self.update_technology_satisfaction(&mut shared_comps)?;
                self.current = None;
            }
        }
        Ok(())
    }

    /// NOTE: Discards technology being researched currently if it exists.
    pub fn start(&mut self, id: TechnologyId) {
        self.current = Some((id, 0));
    }

    pub fn current(&self) -> Option<(TechnologyId, u64)> {
        self.current
    }

    pub fn is_researched(&self, id: TechnologyId) -> bool {
        self.researched.contains(&id)
    }

    pub fn is_transport_unlocked(&self, id: TransportMethodId) -> bool {
        self.unlocked_transport.contains(&id)
    }

    pub fn is_production_unlocked(&self, id: ProductionMethodId) -> bool {
        self.unlocked_production.contains(&id)
    }
}

impl Serializable for Research {
    type Raw = RawResearch;

    fn from_serializable(
        raw: Self::Raw,
        ctx: super::components::ComponentsRef<'_>,
    ) -> anyhow::Result<Self> {
        Ok(Research {
            current: match raw.current {
                Some((id, progress)) => Some((Serializable::from_serializable(id, ctx)?, progress)),
                None => None,
            },
            researched: Serializable::from_serializable(raw.finished, ctx)?,
            unlocked_transport: Serializable::from_serializable(raw.unlocked_transport, ctx)?,
            unlocked_production: Serializable::from_serializable(raw.unlocked_production, ctx)?,
        })
    }

    fn into_serializable(
        self,
        ctx: super::components::ComponentsRef<'_>,
    ) -> anyhow::Result<Self::Raw> {
        Ok(RawResearch {
            current: match self.current {
                Some((id, progress)) => Some((id.into_serializable(ctx)?, progress)),
                None => None,
            },
            finished: self.researched.into_serializable(ctx)?,
            unlocked_transport: self.unlocked_transport.into_serializable(ctx)?,
            unlocked_production: self.unlocked_production.into_serializable(ctx)?,
        })
    }
}
