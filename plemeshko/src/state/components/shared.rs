use anyhow::{anyhow, Result};
use either::Either;

use crate::state::config::{Config, ConfigRepository, FatConfigId};

use super::ComponentId;

pub struct SharedComponent {
    pub configs: ConfigRepository,
}

#[derive(Default)]
pub struct SharedComponents(pub(super) Vec<Option<SharedComponent>>);

impl SharedComponents {
    pub fn get_component_mut(&mut self, id: ComponentId) -> Result<&mut SharedComponent> {
        self.0
            .get_mut(id.0 as usize)
            .ok_or_else(|| anyhow!("Component id out of range: {}", id.0))?
            .as_mut()
            .ok_or_else(|| {
                anyhow!(
                    "Component hasn't finished loading or was unloaded: {}",
                    id.0
                )
            })
    }

    pub fn get_component(&self, id: ComponentId) -> Result<&SharedComponent> {
        self.0
            .get(id.0 as usize)
            .ok_or_else(|| anyhow!("Component id out of range: {}", id.0))?
            .as_ref()
            .ok_or_else(|| {
                anyhow!(
                    "Component hasn't finished loading or was unloaded: {}",
                    id.0
                )
            })
    }

    pub fn get_core(&self) -> Result<&SharedComponent> {
        self.get_component(ComponentId::core())
    }

    pub fn iter_components(&self) -> impl Iterator<Item = (ComponentId, &SharedComponent)> {
        self.0.iter().enumerate().filter_map(|(i, comp)| {
            comp.as_ref()
                .map(|comp| (ComponentId(i.try_into().unwrap()), comp))
        })
    }

    pub fn get_config<C: Config>(&self, id: FatConfigId<C>) -> Result<&C> {
        self.get_component(id.0)?.configs.get(id.1)
    }

    pub fn iter_configs<C: Config>(&self) -> impl Iterator<Item = Result<(FatConfigId<C>, &C)>> {
        self.iter_components()
            .flat_map(|(component_id, component)| match component.configs.iter() {
                Ok(cfgs) => Either::Left(
                    cfgs.map(move |(cfg_id, cfg)| Ok((cfg_id.in_component(component_id), cfg))),
                ),
                Err(e) => Either::Right(std::iter::once(Err(e))),
            })
    }
}
