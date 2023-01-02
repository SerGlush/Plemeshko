use std::{
    borrow::Borrow,
    collections::{hash_map::RawEntryMut, HashMap},
    ops::{AddAssign, SubAssign},
};

use crate::sim::units::ResourceAmount;

use super::ResourceId;

#[derive(Clone)]
pub struct ResourceStorage(HashMap<ResourceId, ResourceAmount>);

impl ResourceStorage {
    pub fn new() -> Self {
        ResourceStorage(HashMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        ResourceStorage(HashMap::with_capacity(capacity))
    }

    pub fn put<I: ?Sized + std::hash::Hash + Eq + ToOwned<Owned = ResourceId>>(
        &mut self,
        id: &I,
        count: ResourceAmount,
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

    pub fn put_many_times(&mut self, res_s: &ResourceStorage, times: i128) {
        for (res_id, res_amount_s) in res_s.iter() {
            let res_amount_t = res_amount_s * times;
            match self.0.raw_entry_mut().from_key(res_id) {
                RawEntryMut::Occupied(mut stored) => stored.get_mut().add_assign(res_amount_t),
                RawEntryMut::Vacant(vacant) => {
                    vacant.insert(res_id.to_owned(), res_amount_t);
                }
            }
        }
    }

    pub fn get(&self, id: &ResourceId) -> Option<ResourceAmount> {
        self.0.get(id).map(Clone::clone)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ResourceId, ResourceAmount)> {
        self.0.iter().map(|(k, v)| (k, *v))
    }

    // pub fn take(&mut self, id: &ResourceId, count: ResourceAmount) -> Result<(), ResourceAmount> {
    //     match self.0.get_mut(id) {
    //         Some(stored) => {
    //             if *stored >= count {
    //                 Ok(stored.sub_assign(count))
    //             } else {
    //                 Err(count - *stored)
    //             }
    //         }
    //         None => Err(count),
    //     }
    // }

    pub fn available(&self, req_res: &ResourceStorage) -> bool {
        for (res_id, req_res_amount) in req_res.iter() {
            let available_amount = self.0.get(res_id).map(Clone::clone).unwrap_or_default();
            if available_amount < req_res_amount {
                return false;
            }
        }
        true
    }

    pub fn sub(&mut self, req_res: &ResourceStorage) -> bool {
        if self.available(req_res) {
            for (res_id, req_res_amount) in req_res.iter() {
                self.0.get_mut(res_id).unwrap().sub_assign(req_res_amount);
            }
            true
        }
        else {
            false
        }
    }

    pub fn sub_unchecked(&mut self, req_res: &ResourceStorage) {
        for (res_id, req_res_amount) in req_res.iter() {
            self.0.get_mut(res_id).unwrap().sub_assign(req_res_amount);
        }
    }

    pub fn available_bounded(&mut self, req_res_singular: &ResourceStorage, mut max: i128) -> i128 {
        debug_assert!(max >= 0);
        for (res_id, req_res_amount_singular) in req_res_singular.iter() {
            let available_amount = self.0.get(res_id).map(Clone::clone).unwrap_or_default();
            max = max.min(available_amount / req_res_amount_singular);
        }
        max
    }

    pub fn sub_bounded(&mut self, req_res_singular: &ResourceStorage, mut max: i128) -> i128 {
        let available = self.available_bounded(req_res_singular, max);
        for (res_id, req_res_amount_singular) in req_res_singular.iter() {
            let given_res_amount = req_res_amount_singular * available;
            self.0.get_mut(res_id).unwrap().sub_assign(given_res_amount);
        }
        available
    }

    pub fn take_bounded(
        &mut self,
        req_res_singular: &ResourceStorage,
        dst: &mut ResourceStorage,
        max: i128,
    ) -> i128 {
        let max = self.sub_bounded(req_res_singular, max);
        for (res_id, req_res_amount_singular) in req_res_singular.iter() {
            let given_res_amount = req_res_amount_singular * max;
            self.0.get_mut(res_id).unwrap().sub_assign(given_res_amount);
            dst.put(res_id, given_res_amount);
        }
        max
    }
}
