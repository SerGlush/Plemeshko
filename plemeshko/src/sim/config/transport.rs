use plegine::config::{Config, ConfigId};
use plegine_derive::Config;

use crate::{sim::transport_group::TransportGroup, units::Volume};

use super::resource::ResourceDelta;

#[derive(Config)]
struct Transport {
    pub max_volume: Volume,
    pub transportation_group: TransportGroup,
    pub cost: ResourceDelta,
}

pub type TransportId = ConfigId<Transport>;
