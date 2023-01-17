use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    env::{
        config::{Config, ConfigId, ConfigLabel, Serializable},
        text::TextId,
    },
    sim::units::{ResourceAmount, ResourceWeight},
};

use super::transport_group::{TransportGroupId, TransportGroupLabel};

#[derive(Deserialize)]
pub struct RawResource {
    pub transport_group: TransportGroupLabel,
    pub transport_weight: ResourceWeight,
}

#[derive(Debug)]
pub struct Resource {
    pub name: TextId,
    pub transport_group: TransportGroupId,
    pub transport_weight: ResourceWeight,
}

pub type ResourceLabel = ConfigLabel<Resource>;
pub type ResourceId = ConfigId<Resource>;

pub type RawResourceMap = HashMap<ResourceLabel, ResourceAmount>;
pub type ResourceMap = HashMap<ResourceId, ResourceAmount>;

#[derive(Serialize, Deserialize)]
pub struct RawResourceIo {
    #[serde(default)]
    pub input: RawResourceMap,
    #[serde(default)]
    pub output: RawResourceMap,
}

#[derive(Default)]
pub struct ResourceIo {
    pub input: ResourceMap,
    pub output: ResourceMap,
}

impl Config for Resource {
    type Raw = RawResource;

    const TAG: &'static str = "resource";

    fn prepare(
        raw: Self::Raw,
        id: ConfigLabel<Self>,
        indexer: &mut crate::env::config::ConfigIndexer,
    ) -> Self {
        Resource {
            name: config_text_id!(id),
            transport_group: indexer.get_or_create_id(raw.transport_group),
            transport_weight: raw.transport_weight,
        }
    }
}

impl Serializable for ResourceIo {
    type Raw = RawResourceIo;

    fn from_serializable(
        raw: RawResourceIo,
        indexer: &mut crate::env::config::ConfigIndexer,
    ) -> Self {
        ResourceIo {
            input: Serializable::from_serializable(raw.input, indexer),
            output: Serializable::from_serializable(raw.output, indexer),
        }
    }

    fn into_serializable(
        self,
        indexer: &mut crate::env::config::ConfigIndexer,
    ) -> anyhow::Result<RawResourceIo> {
        Ok(RawResourceIo {
            input: self.input.into_serializable(indexer)?,
            output: self.output.into_serializable(indexer)?,
        })
    }
}
