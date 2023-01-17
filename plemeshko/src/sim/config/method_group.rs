use serde::Deserialize;

use crate::env::config::{Config, ConfigId, ConfigLabel, Serializable};

use super::method::{MethodId, MethodLabel};

#[derive(Deserialize)]
pub struct RawMethodGroup {
    pub variants: Vec<MethodLabel>,
}

pub struct MethodGroup {
    pub variants: Vec<MethodId>,
}

pub type MethodGroupLabel = ConfigLabel<MethodGroup>;
pub type MethodGroupId = ConfigId<MethodGroup>;

impl Config for MethodGroup {
    type Raw = RawMethodGroup;

    const TAG: &'static str = "method-group";

    fn prepare(
        raw: Self::Raw,
        _id: ConfigLabel<Self>,
        indexer: &mut crate::env::config::ConfigIndexer,
    ) -> Self {
        Self {
            variants: Serializable::from_serializable(raw.variants, indexer),
        }
    }
}
