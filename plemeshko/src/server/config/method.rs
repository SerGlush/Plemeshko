use plegine::config::{Config, ConfigId};
use plegine_derive::Config;

use super::resource::signed_storage::ResourceStorageSigned;

#[derive(Config)]
pub struct Method {
    pub resources: ResourceStorageSigned,
}

pub type MethodId = ConfigId<Method>;
