pub mod signed_storage;
pub mod storage;

use plegine::config::{Config, ConfigId};
use plegine_derive::Config;

use crate::sim::{transport_group::TransportGroup, units::ResourceWeight};

#[derive(Config)]
pub struct Resource {
    pub consumable: bool,
    pub transport_group: TransportGroup,
    pub transport_weight: ResourceWeight,
}

pub type ResourceId = ConfigId<Resource>;
