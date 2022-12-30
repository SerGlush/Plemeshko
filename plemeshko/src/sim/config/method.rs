use std::ops::AddAssign;

use plegine::config::{Config, ConfigId};
use plegine_derive::Config;

use super::resource::ResourceDelta;

#[derive(Config)]
pub struct Method {
    pub delta: ResourceDelta, // negative = import, positive = export
}

pub type MethodId = ConfigId<Method>;

impl Method {
    pub fn accumulate(&self, rs: &mut ResourceDelta) {
        for (id, count) in self.delta.iter() {
            match rs.iter_mut().find(|(other_id, _)| id.eq(other_id)) {
                Some((_, other_count)) => other_count.add_assign(*count),
                None => rs.push((id.clone(), *count)),
            }
        }
    }
}
