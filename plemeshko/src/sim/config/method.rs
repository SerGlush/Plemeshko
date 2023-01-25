use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::env::{
    config::{Config, ConfigId, ConfigLabel, Serializable},
    text::TextId,
    AppEnv,
};

use super::setting_group::{
    self, RawSelectedSetting, SelectedSetting, SettingGroupId, SettingGroupLabel,
};

#[derive(Deserialize)]
pub struct RawMethod {
    pub setting_groups: Vec<SettingGroupLabel>,
}

pub struct Method {
    pub name: TextId,
    pub setting_groups: Vec<SettingGroupId>,
}

pub type MethodLabel = ConfigLabel<Method>;
pub type MethodId = ConfigId<Method>;

#[derive(Serialize, Deserialize)]
pub struct RawSelectedMethod {
    pub label: MethodLabel,
    pub settings: Vec<RawSelectedSetting>,
}

#[derive(Clone)]
pub struct SelectedMethod {
    pub id: MethodId,
    pub settings: Vec<SelectedSetting>,
}

impl SelectedMethod {
    pub fn new(env: &AppEnv, id: ConfigId<Method>, index: Option<usize>) -> Result<SelectedMethod> {
        let method = config_get!(env.configs(), id);
        Ok(SelectedMethod {
            id: id,
            settings: method
                .setting_groups
                .iter()
                .map(|setting_group| SelectedSetting {
                    group: *setting_group,
                    index: index.unwrap_or(0),
                })
                .collect(),
        })
    }
}

impl Config for Method {
    type Raw = RawMethod;

    const TAG: &'static str = "method";

    fn prepare(
        raw: Self::Raw,
        label: ConfigLabel<Self>,
        indexer: &mut crate::env::config::ConfigIndexer,
    ) -> Self {
        Method {
            name: config_text_id!(label),
            setting_groups: Serializable::from_serializable(raw.setting_groups, indexer),
        }
    }
}

impl Serializable for SelectedMethod {
    type Raw = RawSelectedMethod;

    fn from_serializable(raw: Self::Raw, indexer: &mut crate::env::config::ConfigIndexer) -> Self {
        Self {
            id: Serializable::from_serializable(raw.label, indexer),
            settings: Serializable::from_serializable(raw.settings, indexer),
        }
    }

    fn into_serializable(
        self,
        indexer: &mut crate::env::config::ConfigIndexer,
    ) -> anyhow::Result<Self::Raw> {
        Ok(RawSelectedMethod {
            label: self.id.into_serializable(indexer)?,
            settings: self.settings.into_serializable(indexer)?,
        })
    }
}
