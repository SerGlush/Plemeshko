use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::state::{
    components::{ComponentsRef, SharedComponents},
    config::{Config, ConfigsLoadingContext, FatConfigId, FatConfigLabel, Prepare},
    serializable::Serializable,
    text::{FatTextId, TextIdFactory},
};

use super::{
    setting::{Setting, SettingId},
    setting_group::{SettingGroup, SettingGroupId},
};

#[derive(Deserialize)]
pub struct RawMethod {
    pub setting_groups: Vec<FatConfigLabel<SettingGroup>>,
}

pub struct Method {
    pub name: FatTextId,
    pub setting_groups: Vec<SettingGroupId>,
}

pub type MethodLabel = FatConfigLabel<Method>;
pub type MethodId = FatConfigId<Method>;

#[derive(Serialize, Deserialize)]
pub struct RawSelectedMethod {
    pub label: MethodLabel,
    pub settings: Vec<FatConfigLabel<Setting>>,
}

#[derive(Clone)]
pub struct SelectedMethod {
    pub id: MethodId,
    pub settings: Vec<SettingId>,
}

impl SelectedMethod {
    pub fn new(
        shared_comps: &SharedComponents,
        id: MethodId,
        index: Option<usize>,
    ) -> Result<SelectedMethod> {
        let method = shared_comps.get_config(id)?;
        let index = index.unwrap_or(0);
        Ok(SelectedMethod {
            id,
            settings: method
                .setting_groups
                .iter()
                .map(|&setting_group| {
                    match shared_comps.get_config(setting_group)?.settings.get(index) {
                        Some(setting_id) => Ok(*setting_id),
                        None => Err(anyhow!("When creating `SelectedMethod`: Setting index in group out of range: {index}")),
                    }
                })
                .try_collect()?,
        })
    }
}

impl Prepare for RawMethod {
    type Prepared = Method;

    fn prepare(
        self,
        ctx: &mut ConfigsLoadingContext<'_>,
        tif: &mut TextIdFactory,
    ) -> anyhow::Result<Method> {
        let name = tif.create("name").in_component(ctx.component_id());
        tif.with_lock(|tif| {
            Ok(Method {
                name,
                setting_groups: self.setting_groups.prepare(ctx, tif)?,
            })
        })
    }
}

impl Config for Method {
    type Raw = RawMethod;

    const TAG: &'static str = "method";
}

impl Serializable for SelectedMethod {
    type Raw = RawSelectedMethod;

    fn from_serializable(
        raw: Self::Raw,
        ctx: &ComponentsRef<'_>,
    ) -> anyhow::Result<SelectedMethod> {
        Ok(Self {
            id: Serializable::from_serializable(raw.label, ctx)?,
            settings: Serializable::from_serializable(raw.settings, ctx)?,
        })
    }

    fn into_serializable(self, ctx: &ComponentsRef<'_>) -> anyhow::Result<Self::Raw> {
        Ok(RawSelectedMethod {
            label: self.id.into_serializable(ctx)?,
            settings: self.settings.into_serializable(ctx)?,
        })
    }
}
