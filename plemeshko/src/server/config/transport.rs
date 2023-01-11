use plegine::config::{Config, ConfigId};
use plegine_derive::Config;

use crate::server::units::ResourceWeight;

use super::{resource::signed_storage::ResourceStorageSigned, transport_group::TransportGroupId};

#[derive(Config)]
pub struct Transport {
    pub group: TransportGroupId,
    pub capacity: ResourceWeight,
    pub fuel: ResourceStorageSigned,
}

pub type TransportId = ConfigId<Transport>;
