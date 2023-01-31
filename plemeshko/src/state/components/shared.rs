use anyhow::{anyhow, Result};
use either::Either;

use crate::state::config::{Config, ConfigRepository, FatConfigId};

use super::{ComponentId, ComponentSlotId};

pub struct SharedComponent {
    pub configs: ConfigRepository,
}

#[derive(Default)]
pub struct SharedComponents(pub(super) Vec<Option<SharedComponent>>);

impl SharedComponents {
    pub fn component_mut(&mut self, id: ComponentId) -> Result<&mut SharedComponent> {
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

    pub fn component(&self, id: ComponentId) -> Result<&SharedComponent> {
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

    pub fn component_slot(&self, id: ComponentSlotId) -> Result<&Option<SharedComponent>> {
        self.0
            .get(id.0)
            .ok_or_else(|| anyhow!("Component id out of range: {}", id.0))
    }

    pub fn component_slot_mut(
        &mut self,
        id: ComponentSlotId,
    ) -> Result<&mut Option<SharedComponent>> {
        self.0
            .get_mut(id.0)
            .ok_or_else(|| anyhow!("Component id out of range: {}", id.0))
    }

    pub fn core(&self) -> Result<&SharedComponent> {
        self.component(ComponentId::core())
    }

    pub fn iter_components(&self) -> impl Iterator<Item = (ComponentId, &SharedComponent)> {
        self.0.iter().enumerate().filter_map(|(i, comp)| {
            comp.as_ref()
                .map(|comp| (ComponentId(i.try_into().unwrap()), comp))
        })
    }

    pub fn iter_components_mut(
        &mut self,
    ) -> impl Iterator<Item = (ComponentId, &mut SharedComponent)> {
        self.0.iter_mut().enumerate().filter_map(|(i, comp)| {
            comp.as_mut()
                .map(|comp| (ComponentId(i.try_into().unwrap()), comp))
        })
    }

    pub fn config<C: Config>(&self, id: FatConfigId<C>) -> Result<&C> {
        self.component(id.0)?.configs.storage()?.get(id.1)
    }

    pub fn config_mut<C: Config>(&mut self, id: FatConfigId<C>) -> Result<&mut C> {
        self.component_mut(id.0)?
            .configs
            .storage_mut()?
            .into_mut(id.1)
    }

    pub fn iter_configs<C: Config>(&self) -> impl Iterator<Item = Result<(FatConfigId<C>, &C)>> {
        self.iter_components()
            .flat_map(
                |(component_id, component)| match component.configs.storage() {
                    Ok(storage) => Either::Right(
                        storage
                            .iter()
                            .map(move |(cfg_id, cfg)| Ok((cfg_id.in_component(component_id), cfg))),
                    ),
                    Err(e) => Either::Left(std::iter::once(Err(e))),
                },
            )
    }

    pub fn iter_configs_mut<C: Config>(
        &mut self,
    ) -> impl Iterator<Item = Result<(FatConfigId<C>, &mut C)>> {
        self.iter_components_mut()
            .flat_map(
                |(component_id, component)| match component.configs.storage_mut() {
                    Ok(storage) => Either::Right(
                        storage
                            .into_iter()
                            .map(move |(cfg_id, cfg)| Ok((cfg_id.in_component(component_id), cfg))),
                    ),
                    Err(e) => Either::Left(std::iter::once(Err(e))),
                },
            )
    }
}
