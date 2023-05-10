use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::{
    state::{
        components::{ComponentsRef, SharedComponents},
        config::{
            Config, ConfigsLoadingContext, FatConfigId, FatConfigLabel, Info, Prepare, RawInfo,
        },
        serializable::Serializable,
        text::TextIdFactory,
    },
    util::cor::Cor,
};

use super::{
    resource::ResourceMap,
    setting::{Setting, SettingId},
    setting_group::{SettingGroup, SettingGroupId},
};

#[derive(Deserialize)]
pub struct RawProductionMethod {
    #[serde(flatten)]
    pub info: RawInfo,
    pub setting_groups: Vec<FatConfigLabel<SettingGroup>>,
    #[serde(default)]
    pub initially_unlocked: bool,
}

#[derive(Debug)]
pub struct ProductionMethod {
    pub info: Info,
    pub setting_groups: Vec<SettingGroupId>,
    pub initially_unlocked: bool,
}

pub type ProductionMethodId = FatConfigId<ProductionMethod>;

#[derive(Serialize, Deserialize)]
pub struct RawFixedProductionMethod {
    pub label: FatConfigLabel<ProductionMethod>,
    pub settings: Vec<FatConfigLabel<Setting>>,
}

#[derive(Clone)]
pub struct FixedProductionMethod {
    pub id: ProductionMethodId,
    pub settings: Vec<SettingId>,
}

impl FixedProductionMethod {
    pub fn new(
        shared_comps: &SharedComponents,
        id: ProductionMethodId,
        index: Option<usize>,
    ) -> Result<FixedProductionMethod> {
        let method = shared_comps.config(id)?;
        let index = index.unwrap_or(0);
        Ok(FixedProductionMethod {
            id,
            settings: method
                .setting_groups
                .iter()
                .map(|&setting_group| {
                    match shared_comps.config(setting_group)?.settings.get(index) {
                        Some(setting_id) => Ok(*setting_id),
                        None => Err(anyhow!("When creating `FixedProductionMethod`: Setting index in group out of range: {index}")),
                    }
                })
                .try_collect()?,
        })
    }

    pub fn accumulate_cost(
        &self,
        shared_comps: &SharedComponents,
        cost: &mut ResourceMap,
    ) -> Result<()> {
        for &setting_id in self.settings.iter() {
            let setting = shared_comps.config(setting_id)?;
            cost.cor_put_all(&setting.cost);
        }
        Ok(())
    }
}

impl Prepare for RawProductionMethod {
    type Prepared = ProductionMethod;

    fn prepare(
        self,
        ctx: &mut ConfigsLoadingContext<'_>,
        tif: &mut TextIdFactory,
    ) -> anyhow::Result<ProductionMethod> {
        let info = self.info.prepare(ctx, tif)?;
        tif.with_lock(|tif| {
            Ok(ProductionMethod {
                info,
                setting_groups: self.setting_groups.prepare(ctx, tif)?,
                initially_unlocked: self.initially_unlocked,
            })
        })
    }
}

impl Config for ProductionMethod {
    type Raw = RawProductionMethod;

    const TAG: &'static str = "production-method";
}

impl Serializable for FixedProductionMethod {
    type Raw = RawFixedProductionMethod;

    fn from_serializable(
        raw: Self::Raw,
        ctx: ComponentsRef<'_>,
    ) -> anyhow::Result<FixedProductionMethod> {
        Ok(Self {
            id: Serializable::from_serializable(raw.label, ctx)?,
            settings: Serializable::from_serializable(raw.settings, ctx)?,
        })
    }

    fn into_serializable(self, ctx: ComponentsRef<'_>) -> anyhow::Result<Self::Raw> {
        Ok(RawFixedProductionMethod {
            label: self.id.into_serializable(ctx)?,
            settings: self.settings.into_serializable(ctx)?,
        })
    }
}
