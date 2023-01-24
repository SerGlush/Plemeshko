use serde::Deserialize;

use crate::{
    env::{
        config::{Config, ConfigId, ConfigLabel, Serializable},
        text::TextId,
    },
    sim::units::ResourceWeight,
};

use super::{
    resource::{RawResourceIo, ResourceIo},
    transport_group::{TransportGroupId, TransportGroupLabel},
};

#[derive(Deserialize)]
pub struct RawTransport {
    pub group: TransportGroupLabel,
    pub capacity: ResourceWeight,
    pub fuel: RawResourceIo,
}

pub struct Transport {
    pub name: TextId,
    pub group: TransportGroupId,
    pub capacity: ResourceWeight,
    pub fuel: ResourceIo,
}

pub type TransportLabel = ConfigLabel<Transport>;
pub type TransportId = ConfigId<Transport>;

impl Config for Transport {
    type Raw = RawTransport;

    const TAG: &'static str = "transport";

    fn prepare(
        raw: Self::Raw,
        label: ConfigLabel<Self>,
        indexer: &mut crate::env::config::ConfigIndexer,
    ) -> Self {
        Transport {
            name: config_text_id!(label),
            group: indexer.get_or_create_id(raw.group),
            capacity: raw.capacity,
            fuel: Serializable::from_serializable(raw.fuel, indexer),
        }
    }
}
