pub mod storage;

use plegine::config::ConfigId;
use plegine_derive::Config;
use serde::Deserialize;

use crate::sim::units::ResourceWeight;

use super::transport_group::TransportGroupId;

#[derive(Config, Debug, Deserialize)]
pub struct Resource {
    pub transport_group: TransportGroupId,
    pub transport_weight: ResourceWeight,
}

pub type ResourceId = ConfigId<Resource>;
