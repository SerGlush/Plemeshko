use plegine::config::ConfigId;
use plegine_derive::Config;
use serde::Deserialize;

use super::method::MethodId;

#[derive(Config, Deserialize)]
pub struct MethodGroup {
    pub variants: Vec<MethodId>,
}

pub type MethodGroupId = ConfigId<MethodId>;
