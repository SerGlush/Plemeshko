use plegine::config::{Config, ConfigId};
use plegine_derive::Config;

#[derive(Config)]
pub struct TransportGroup {}

pub type TransportGroupId = ConfigId<TransportGroup>;
