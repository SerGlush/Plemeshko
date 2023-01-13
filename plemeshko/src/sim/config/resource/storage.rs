use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::sim::units::ResourceAmount;

use super::ResourceId;

pub use crate::util::cor::*;

pub type ResourceMap = HashMap<ResourceId, ResourceAmount>;

#[derive(Clone, Default, Serialize, Deserialize)]
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
