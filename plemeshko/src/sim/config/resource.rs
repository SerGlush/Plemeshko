use plegine::config::{Config, ConfigId};
use plegine_derive::Config;

#[derive(Config)]
pub struct Resource {
    pub dispensable: bool,
}

pub type ResourceCount = i128;

pub type ResourceId = ConfigId<Resource>;
pub type ResourceVec = Vec<(ResourceId, ResourceCount)>;
