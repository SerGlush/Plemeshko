use std::ops::AddAssign;

use plegine::config::{Config, ConfigId};
use plegine_derive::Config;

use super::resource::ResourceVec;

#[derive(Config)]
struct Method {
    pub delta: ResourceVec,
}

pub type MethodId = ConfigId<Method>;

impl Method {
    pub fn apply(&self, rs: &mut ResourceVec) {
        for (id, count) in self.delta.iter() {
            match rs.iter_mut().find(|(other_id, _)| id.eq(other_id)) {
                Some((_, other_count)) => other_count.add_assign(*count),
                None => rs.push((id.clone(), *count)),
            }
        }
    }
}
