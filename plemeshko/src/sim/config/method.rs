use plegine::config::ConfigId;
use plegine_derive::Config;
use serde::{Deserialize, Serialize};

use super::setting_group::{SelectedSetting, SettingGroupId};

#[derive(Config, Deserialize)]
pub struct Method {
    pub setting_groups: Vec<SettingGroupId>,
}

pub type MethodId = ConfigId<Method>;

#[derive(Clone, Serialize, Deserialize)]
pub struct SelectedMethod {
    pub id: MethodId,
    pub settings: Vec<SelectedSetting>,
}
