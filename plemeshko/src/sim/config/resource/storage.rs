use std::{
    borrow::Borrow,
    collections::{hash_map::RawEntryMut, HashMap},
    ops::{AddAssign, SubAssign},
};

use crate::sim::units::ResourceAmount;

use super::ResourceId;

pub type ResourceStorage = HashMap<ResourceId, ResourceAmount>;

pub use crate::cor::*;
