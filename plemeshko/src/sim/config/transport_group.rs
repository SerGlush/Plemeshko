use serde::Deserialize;

use crate::env::config::{Config, ConfigId, ConfigLabel};

#[derive(Deserialize)]
pub struct RawTransportGroup {}

pub struct TransportGroup {}

pub type TransportGroupLabel = ConfigLabel<TransportGroup>;
pub type TransportGroupId = ConfigId<TransportGroup>;

impl Config for TransportGroup {
    type Raw = RawTransportGroup;

    const TAG: &'static str = "transport-group";

    fn prepare(
        _raw: Self::Raw,
        _label: ConfigLabel<Self>,
        _indexer: &mut crate::env::config::ConfigIndexer,
    ) -> Self {
        TransportGroup {}
    }
}
