use plegine::config::ConfigId;
use plegine_derive::Config;
use serde::Deserialize;

#[derive(Config, Deserialize)]
pub struct TransportGroup {}

pub type TransportGroupId = ConfigId<TransportGroup>;
