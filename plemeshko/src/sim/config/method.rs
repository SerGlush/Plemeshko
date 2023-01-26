use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::state::{
    components::SharedComponents,
    config::{Config, ConfigsLoadingContext, FatConfigId, FatConfigLabel, Prepare},
    serializable::{Serializable, SerializationContext},
    text::{FatTextId, TextIdFactory},
};

use super::setting_group::{RawSelectedSetting, SelectedSetting, SettingGroup, SettingGroupId};

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
    pub settings: Vec<RawSelectedSetting>,
}

#[derive(Clone)]
pub struct SelectedMethod {
    pub id: MethodId,
    pub settings: Vec<SelectedSetting>,
}

impl SelectedMethod {
    pub fn new(
        shared_comps: &SharedComponents,
        id: MethodId,
        index: Option<usize>,
    ) -> Result<SelectedMethod> {
        let method = shared_comps.get_config(id)?;
        Ok(SelectedMethod {
            id,
            settings: method
                .setting_groups
                .iter()
                .map(|setting_group| SelectedSetting {
                    group: *setting_group,
                    index: index.unwrap_or(0),
                })
                .collect(),
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
        ctx: &mut SerializationContext<'_>,
    ) -> anyhow::Result<SelectedMethod> {
        Ok(Self {
            id: Serializable::from_serializable(raw.label, ctx)?,
            settings: Serializable::from_serializable(raw.settings, ctx)?,
        })
    }

    fn into_serializable(self, ctx: &SerializationContext<'_>) -> anyhow::Result<Self::Raw> {
        Ok(RawSelectedMethod {
            label: self.id.into_serializable(ctx)?,
            settings: self.settings.into_serializable(ctx)?,
        })
    }
}
