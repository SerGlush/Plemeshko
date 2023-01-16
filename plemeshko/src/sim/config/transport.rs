use plegine::config::ConfigId;
use plegine_derive::Config;
use serde::Deserialize;

use crate::{env::text::TextId, sim::units::ResourceWeight};

use super::{resource::storage::ResourceIo, transport_group::TransportGroupId};

#[derive(Config, Deserialize)]
pub struct Transport {
    pub name: TextId,
    pub group: TransportGroupId,
    pub capacity: ResourceWeight,
    pub fuel: ResourceIo,
}

pub type TransportId = ConfigId<Transport>;
