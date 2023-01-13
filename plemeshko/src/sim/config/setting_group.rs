use std::collections::HashMap;

use plegine::config::ConfigId;
use plegine_derive::Config;
use serde::{Deserialize, Serialize};

use crate::sim::units::{ResourceAmount, Ticks};

use super::resource::ResourceId;

#[derive(Deserialize)]
pub struct Setting {
    pub input: HashMap<ResourceId, ResourceAmount>,
    pub output: HashMap<ResourceId, ResourceAmount>,
    #[serde(default)]
    pub time_to_complete: Ticks,
}

#[derive(Config, Deserialize)]
pub struct SettingGroup {
    pub settings: Vec<Setting>,
}

pub type SettingGroupId = ConfigId<SettingGroup>;

#[derive(Clone, Serialize, Deserialize)]
pub struct SelectedSetting {
    pub group_id: SettingGroupId,
    pub index: usize,
}
