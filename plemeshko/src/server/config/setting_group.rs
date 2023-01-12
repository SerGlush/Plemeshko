use plegine::{
    config::{Config, ConfigId},
    json::FromValue,
};
use plegine_derive::{Config, FromValue};

use crate::server::units::Ticks;

use super::resource::signed_storage::ResourceStorageSigned;

#[derive(FromValue)]
pub struct Setting {
    pub resources: Vec<ResourceStorageSigned>,
    pub time_to_complete: Option<Ticks>,
}

#[derive(Config)]
pub struct SettingGroup {
    pub settings: Vec<Setting>,
}

pub type SettingGroupId = ConfigId<SettingGroup>;
