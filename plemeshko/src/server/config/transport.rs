use std::collections::HashMap;

use plegine::config::{Config, ConfigId};
use plegine_derive::Config;

use crate::server::{transport_group::TransportGroup, units::ResourceWeight};

use super::resource::signed_storage::ResourceStorageSigned;

#[derive(Config)]
pub struct Transport {
    pub group: TransportGroup,
    pub capacity: ResourceWeight,
    pub fuel: ResourceStorageSigned,
}

pub type TransportId = ConfigId<Transport>;

// todo: make an array ? (|TG| is small)
pub type TransportMap<T> = HashMap<TransportGroup, T>;
