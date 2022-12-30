use plegine::config::{Config, ConfigId};
use plegine_derive::Config;

use crate::{
    sim::{
        error::{SimError, SimResult},
        transport_group::TransportGroup,
        Sim,
    },
    units::{Density, Mass},
};

#[derive(Config)]
pub struct Resource {
    pub dispensable: bool,
    pub transportation_group: TransportGroup,
    pub density: Density,
}

pub type ResourceId = ConfigId<Resource>;
pub type ResourceDelta = Vec<(ResourceId, Mass)>;

pub fn delta_resources<'a>(
    sim: &'a Sim,
    delta: &'a ResourceDelta,
) -> impl Iterator<Item = SimResult<(&'a Resource, Mass)>> {
    delta.iter().map(|(id, mass)| {
        let resource = sim
            .configs
            .get(id)
            .map_err(SimError::ConfigRetrievalFailed)?;
        Ok((resource, mass.clone()))
    })
}
