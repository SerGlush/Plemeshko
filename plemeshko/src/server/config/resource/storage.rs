use std::{
    collections::HashMap,
};

use crate::server::units::ResourceAmount;

use super::ResourceId;

pub type ResourceStorage = HashMap<ResourceId, ResourceAmount>;

pub use crate::cor::*;
