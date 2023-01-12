use plegine::{
    config::{Config, ConfigId},
    json::FromValue,
};
use plegine_derive::{Config, FromValue};

use super::resource::signed_storage::ResourceStorageSigned;

#[derive(FromValue)]
pub struct Setting {
    pub resources: Vec<ResourceStorageSigned>,
}

#[derive(Config)]
pub struct SettingGroup {
    pub settings: Vec<Setting>,
}

pub type SettingGroupId = ConfigId<SettingGroup>;
