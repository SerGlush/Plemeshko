use serde::{Deserialize, Serialize};

use crate::{
    env::config::{Config, ConfigId, ConfigLabel, Serializable},
    sim::units::Ticks,
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

pub type SettingGroupLabel = ConfigLabel<SettingGroup>;
pub type SettingGroupId = ConfigId<SettingGroup>;

#[derive(Serialize, Deserialize)]
pub struct RawSelectedSetting {
    pub group: SettingGroupLabel,
    pub index: usize,
}

#[derive(Clone)]
pub struct SelectedSetting {
    pub group: SettingGroupId,
    pub index: usize,
}

impl Config for SettingGroup {
    type Raw = RawSettingGroup;

    const TAG: &'static str = "setting-group";

    fn prepare(
        raw: Self::Raw,
        _id: ConfigLabel<Self>,
        indexer: &mut crate::env::config::ConfigIndexer,
    ) -> Self {
        SettingGroup {
            settings: Serializable::from_serializable(raw.settings, indexer),
        }
    }
}

impl Serializable for Setting {
    type Raw = RawSetting;

    fn from_serializable(raw: RawSetting, indexer: &mut crate::env::config::ConfigIndexer) -> Self {
        Setting {
            resource_io: Serializable::from_serializable(raw.resource_io, indexer),
            time_to_complete: raw.time_to_complete,
        }
    }

    fn into_serializable(
        self,
        indexer: &mut crate::env::config::ConfigIndexer,
    ) -> anyhow::Result<RawSetting> {
        Ok(RawSetting {
            resource_io: self.resource_io.into_serializable(indexer)?,
            time_to_complete: self.time_to_complete,
        })
    }
}

impl Serializable for SelectedSetting {
    type Raw = RawSelectedSetting;

    fn from_serializable(raw: Self::Raw, indexer: &mut crate::env::config::ConfigIndexer) -> Self {
        SelectedSetting {
            group: Serializable::from_serializable(raw.group, indexer),
            index: raw.index,
        }
    }

    fn into_serializable(
        self,
        indexer: &mut crate::env::config::ConfigIndexer,
    ) -> anyhow::Result<Self::Raw> {
        Ok(RawSelectedSetting {
            group: self.group.into_serializable(indexer)?,
            index: self.index,
        })
    }
}
