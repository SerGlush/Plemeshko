use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    sim::units::Ticks,
    state::{
        components::SharedComponents,
        config::{Config, FatConfigId, FatConfigLabel, Prepare},
        text::FatTextId,
    },
};

use super::{
    resource::{RawResourceIo, RawResourceMap, ResourceIo, ResourceMap},
    setting_group::{SettingGroup, SettingGroupId},
};

#[derive(Serialize, Deserialize)]
pub struct RawSetting {
    pub group: FatConfigLabel<SettingGroup>,
    #[serde(flatten)]
    pub resource_io: RawResourceIo,
    #[serde(default)]
    pub cost: RawResourceMap,
    #[serde(default)]
    pub time_to_complete: Ticks,
}

#[derive(Debug)]
pub struct Setting {
    pub name: FatTextId,
    pub group: SettingGroupId,
    pub resource_io: ResourceIo,
    pub cost: ResourceMap,
    pub time_to_complete: Ticks,
}

pub type SettingId = FatConfigId<Setting>;

impl Setting {
    pub fn group<'a>(&self, shared_comps: &'a SharedComponents) -> Result<&'a SettingGroup> {
        shared_comps.config(self.group)
    }
}

impl Prepare for RawSetting {
    type Prepared = Setting;

    fn prepare(
        self,
        ctx: &mut crate::state::config::ConfigsLoadingContext<'_>,
        tif: &mut crate::state::text::TextIdFactory,
    ) -> anyhow::Result<Self::Prepared> {
        let name = tif.create("name").in_component(ctx.this_component.id());
        tif.with_lock(|tif| {
            Ok(Setting {
                name,
                group: self.group.prepare(ctx, tif)?,
                resource_io: self.resource_io.prepare(ctx, tif)?,
                cost: self.cost.prepare(ctx, tif)?,
                time_to_complete: self.time_to_complete,
            })
        })
    }
}

impl Config for Setting {
    type Raw = RawSetting;

    const TAG: &'static str = "setting";
}
