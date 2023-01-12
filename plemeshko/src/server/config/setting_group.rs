use egui::epaint::ahash::HashMap;
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
    pub time_to_complete: Option<Ticks>,
}

#[derive(Config)]
pub struct SettingGroup {
    pub settings: Vec<Setting>,
}

pub type SettingGroupId = ConfigId<SettingGroup>;
