use plegine::config::{Config, ConfigId};
use plegine_derive::Config;

use super::method::MethodId;

#[derive(Config)]
struct MethodGroup {
    pub variants: Vec<MethodId>,
}

pub type MethodGroupId = ConfigId<MethodId>;
