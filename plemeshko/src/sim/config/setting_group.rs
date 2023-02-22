use anyhow::Result;
use serde::Deserialize;

use crate::state::{
    components::{ComponentIndexer, SharedComponents},
    config::{Config, ConfigsLoadingContext, FatConfigId, Prepare},
    text::TextIdFactory,
};

use super::setting::{Setting, SettingId};

#[derive(Deserialize)]
pub struct RawSettingGroup {}

#[derive(Debug)]
pub struct SettingGroup {
    pub settings: Vec<SettingId>,
}

pub type SettingGroupId = FatConfigId<SettingGroup>;

impl Prepare for RawSettingGroup {
    type Prepared = SettingGroup;

    fn prepare(
        self,
        _ctx: &mut ConfigsLoadingContext<'_>,
        _tif: &mut TextIdFactory,
    ) -> anyhow::Result<Self::Prepared> {
        Ok(SettingGroup {
            settings: Vec::new(),
        })
    }
}

impl Config for SettingGroup {
    type Raw = RawSettingGroup;

    const TAG: &'static str = "setting-group";

    fn finalize(indexer: &ComponentIndexer, shared_comps: &mut SharedComponents) -> Result<()> {
        // clear all setting groups
        for setting_group in shared_comps.iter_configs_mut::<SettingGroup>() {
            let setting_group = setting_group?.1;
            setting_group.settings.clear();
        }

        // for all components - find all settings and push to respective groups
        let component_slot_ids = indexer.indices();
        for component_slot_id in component_slot_ids {
            let component_setting_ids = match shared_comps.component_slot(component_slot_id)? {
                Some(component) => component.configs.indexer::<Setting>()?.indices::<Setting>(),
                None => continue,
            };
            let component_id = component_slot_id.assume_occupied();
            for component_setting_id in component_setting_ids {
                let setting_group_id = shared_comps
                    .component_slot(component_slot_id)
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .configs
                    .storage::<Setting>()?
                    .get(component_setting_id)?
                    .group;
                shared_comps
                    .config_mut(setting_group_id)?
                    .settings
                    .push(FatConfigId(component_id, component_setting_id));
            }
        }
        Ok(())
    }
}
