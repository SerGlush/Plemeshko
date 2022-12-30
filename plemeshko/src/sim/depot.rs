use std::{
    borrow::Borrow,
    collections::{hash_map::RawEntryMut, HashMap},
    ops::{AddAssign, SubAssign},
};

use crate::units::Mass;

use super::config::resource::ResourceId;

pub struct Depot(HashMap<ResourceId, Mass>);

impl Depot {
    pub fn new() -> Depot {
        Depot(HashMap::new())
    }

    pub fn put<I: ?Sized + std::hash::Hash + Eq + ToOwned<Owned = ResourceId>>(
        &mut self,
        id: &I,
        count: Mass,
    ) where
        ResourceId: Borrow<I>,
    {
        match self.0.raw_entry_mut().from_key(id) {
            RawEntryMut::Occupied(mut stored) => stored.get_mut().add_assign(count),
            RawEntryMut::Vacant(vacant) => {
                vacant.insert(id.to_owned(), count);
            }
        }
    }

    pub fn get(&self, id: &ResourceId) -> Option<Mass> {
        self.0.get(id).map(Clone::clone)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ResourceId, Mass)> {
        self.0.iter().map(|(k, v)| (k, *v))
    }

    pub fn take(&mut self, id: &ResourceId, count: Mass) -> Result<(), Mass> {
        match self.0.get_mut(id) {
            Some(stored) => {
                if *stored >= count {
                    Ok(stored.sub_assign(count))
                } else {
                    Err(count - *stored)
                }
            }
            None => Err(count),
        }
    }
}
