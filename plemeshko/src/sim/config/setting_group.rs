use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::state::{
    components::SharedComponents,
    config::{Config, ConfigsLoadingContext, FatConfigId, FatConfigLabel, Prepare},
    serializable::{Serializable, SerializationContext},
    text::TextIdFactory,
};

use super::setting::{Setting, SettingId};

#[derive(Deserialize)]
pub struct RawSettingGroup {
    pub settings: Vec<FatConfigLabel<Setting>>,
}

pub struct SettingGroup {
    pub settings: Vec<SettingId>,
}

pub type SettingGroupId = FatConfigId<SettingGroup>;

#[derive(Serialize, Deserialize)]
pub struct RawSelectedSetting {
    pub group: FatConfigLabel<SettingGroup>,
    pub index: usize,
}

#[derive(Clone)]
pub struct SelectedSetting {
    pub group: SettingGroupId,
    pub index: usize,
}

impl SettingGroup {
    pub fn setting<'a>(
        &self,
        shared_comps: &'a SharedComponents,
        index: usize,
    ) -> Result<&'a Setting> {
        shared_comps.get_config(self.settings[index])
    }
}

impl Prepare for RawSettingGroup {
    type Prepared = SettingGroup;

    fn prepare(
        self,
        ctx: &mut ConfigsLoadingContext<'_>,
        tif: &mut TextIdFactory,
    ) -> anyhow::Result<Self::Prepared> {
        Ok(SettingGroup {
            settings: tif.with_branch("settings", |tif| self.settings.prepare(ctx, tif))?,
        })
    }
}

impl Config for SettingGroup {
    type Raw = RawSettingGroup;

    const TAG: &'static str = "setting-group";
}

impl Serializable for SelectedSetting {
    type Raw = RawSelectedSetting;

    fn from_serializable(raw: Self::Raw, ctx: &mut SerializationContext<'_>) -> Result<Self> {
        Ok(SelectedSetting {
            group: Serializable::from_serializable(raw.group, ctx)?,
            index: raw.index,
        })
    }

    fn into_serializable(self, ctx: &SerializationContext<'_>) -> anyhow::Result<Self::Raw> {
        Ok(RawSelectedSetting {
            group: self.group.into_serializable(ctx)?,
            index: self.index,
        })
    }
}
