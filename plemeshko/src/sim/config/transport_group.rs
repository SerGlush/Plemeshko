use serde::Deserialize;

use crate::env::{config::{Config, ConfigId, ConfigLabel}, text::TextId};

#[derive(Deserialize)]
pub struct RawTransportGroup {}

pub struct TransportGroup {
    pub name: TextId,
}

pub type TransportGroupLabel = ConfigLabel<TransportGroup>;
pub type TransportGroupId = ConfigId<TransportGroup>;

impl Config for TransportGroup {
    type Raw = RawTransportGroup;

    const TAG: &'static str = "transport-group";

    fn prepare(
        _raw: Self::Raw,
        label: ConfigLabel<Self>,
        _indexer: &mut crate::env::config::ConfigIndexer,
    ) -> Self {
        TransportGroup {
            name: config_text_id!(label),
        }
    }
}
