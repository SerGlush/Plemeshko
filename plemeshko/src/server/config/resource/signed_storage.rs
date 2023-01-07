use plegine::json;

use crate::server::units::ResourceAmount;

use super::{
    storage::{Cor, ResourceStorage},
    ResourceId,
};

pub struct ResourceStorageSigned {
    pub positive: ResourceStorage,
    pub negative: ResourceStorage,
}

impl json::FromValue for ResourceStorageSigned {
    fn from_value(value: json::Value) -> json::ParseResult<Self> {
        let deltas = json::Array::from_value(value)?;
        let mut positive = ResourceStorage::with_capacity(deltas.len() / 2);
        let mut negative = ResourceStorage::with_capacity(deltas.len() / 2);
        for delta in deltas.into_iter() {
            let (id, delta) = <(ResourceId, ResourceAmount)>::from_value(delta)?;
            if delta < ResourceAmount::default() {
                negative.cor_put(&id, -delta);
            } else {
                positive.cor_put(&id, delta);
            }
        }
        Ok(ResourceStorageSigned { positive, negative })
    }
}

impl ResourceStorageSigned {
    pub fn new() -> ResourceStorageSigned {
        ResourceStorageSigned {
            positive: ResourceStorage::new(),
            negative: ResourceStorage::new(),
        }
    }
}

//     // pub fn accumulate(&mut self, delta: &ResourcesDelta) {
//     //     for (id, count) in delta.positive.iter() {
//     //         match self.positive.iter_mut().find(|(other_id, _)| id.eq(other_id)) {
//     //             Some((_, other_count)) => other_count.add_assign(*count),
//     //             None => self.positive.push((id.clone(), *count)),
//     //         }
//     //     }
//     //     for (id, count) in delta.negative.iter() {
//     //         match self.negative.iter_mut().find(|(other_id, _)| id.eq(other_id)) {
//     //             Some((_, other_count)) => other_count.add_assign(*count),
//     //             None => self.negative.push((id.clone(), *count)),
//     //         }
//     //     }
//     // }

//     pub fn iter_positive<'a>(
//         &'a self,
//         sim: &'a Sim,
//     ) -> impl Iterator<Item = SimResult<(&'a Resource, ResourceAmount)>> {
//         self.positive.iter().map(|(id, amount)| {
//             let resource = sim
//                 .configs
//                 .get(id)
//                 .map_err(SimError::ConfigRetrievalFailed)?;
//             Ok((resource, amount.clone()))
//         })
//     }

//     pub fn iter_negative<'a>(
//         &'a self,
//         sim: &'a Sim,
//     ) -> impl Iterator<Item = SimResult<(&'a Resource, ResourceAmount)>> {
//         self.negative.iter().map(|(id, amount)| {
//             let resource = sim
//                 .configs
//                 .get(id)
//                 .map_err(SimError::ConfigRetrievalFailed)?;
//             Ok((resource, amount.clone()))
//         })
//     }

//     pub fn iter<'a>(
//         &'a self,
//         sim: &'a Sim,
//     ) -> impl Iterator<Item = SimResult<(&'a Resource, ResourceAmount)>> {
//         let positive = self.iter_positive(sim);
//         let negative = self.negative.iter().map(|(id, amount)| {
//             let resource = sim
//                 .configs
//                 .get(id)
//                 .map_err(SimError::ConfigRetrievalFailed)?;
//             Ok((resource, -amount.clone()))
//         });
//         positive.chain(negative)
//     }
// }
