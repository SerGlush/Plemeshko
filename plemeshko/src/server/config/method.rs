use plegine::config::{Config, ConfigId};
use plegine_derive::Config;

use super::setting_group::SettingGroupId;

#[derive(Config)]
pub struct Method {
    pub setting_groups: Vec<SettingGroupId>,
}

pub type MethodId = ConfigId<Method>;
