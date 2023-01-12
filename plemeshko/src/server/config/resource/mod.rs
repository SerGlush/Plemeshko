pub mod signed_storage;
pub mod storage;

use plegine::config::{Config, ConfigId};
use plegine_derive::Config;

use crate::server::units::ResourceWeight;

use super::transport_group::TransportGroupId;

#[derive(Config)]
pub struct Resource {
    pub transport_group: TransportGroupId,
    pub transport_weight: ResourceWeight,
}

pub type ResourceId = ConfigId<Resource>;
