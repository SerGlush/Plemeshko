use std::collections::HashMap;

use serde::Deserialize;

use crate::sim::units::ResourceAmount;

use super::ResourceId;

pub use crate::util::cor::*;

pub type ResourceMap = HashMap<ResourceId, ResourceAmount>;

#[derive(Deserialize, Default)]
pub struct ResourceIo {
    #[serde(default)]
    pub input: ResourceMap,
    #[serde(default)]
    pub output: ResourceMap,
}

impl ResourceIo {
    pub fn new() -> ResourceIo {
        Self::default()
    }
}
