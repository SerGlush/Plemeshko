use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    sim::units::Ticks,
    state::{
        config::{Config, ConfigsLoadingContext, FatConfigId, FatConfigLabel, Prepare},
        serializable::{Serializable, SerializationContext},
        text::{FatTextId, TextIdFactory},
    },
};

use super::resource::{RawResourceIo, ResourceIo};

#[derive(Serialize, Deserialize)]
pub struct RawSetting {
    #[serde(flatten)]
    pub resource_io: RawResourceIo,
    #[serde(default)]
    pub time_to_complete: Ticks,
}

pub struct Setting {
    pub name: FatTextId,
    pub resource_io: ResourceIo,
    pub time_to_complete: Ticks,
}

#[derive(Deserialize)]
pub struct RawSettingGroup {
    pub settings: Vec<RawSetting>,
}

pub struct SettingGroup {
    pub settings: Vec<Setting>,
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

impl Prepare for RawSetting {
    type Prepared = Setting;

    fn prepare(
        self,
        ctx: &mut ConfigsLoadingContext<'_>,
        tif: &mut TextIdFactory,
    ) -> anyhow::Result<Self::Prepared> {
        let name = tif.create("name").in_component(ctx.component_id());
        tif.with_lock(|tif| {
            Ok(Setting {
                name,
                resource_io: self.resource_io.prepare(ctx, tif)?,
                time_to_complete: self.time_to_complete,
            })
        })
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
