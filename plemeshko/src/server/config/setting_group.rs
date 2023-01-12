use std::collections::HashMap;

use plegine::{
    config::{Config, ConfigId},
    json::FromValue,
};
use plegine_derive::{Config, FromValue};

use crate::server::units::{ResourceAmount, Ticks};

use super::resource::ResourceId;

#[derive(FromValue)]
pub struct Setting {
    pub input: HashMap<ResourceId, ResourceAmount>,
    pub output: HashMap<ResourceId, ResourceAmount>,
    #[default]
    pub time_to_complete: Ticks,
}

#[derive(Config)]
pub struct SettingGroup {
    pub settings: Vec<Setting>,
}

pub type SettingGroupId = ConfigId<SettingGroup>;

pub struct SelectedSetting {
    pub group_id: SettingGroupId,
    pub index: usize,
}