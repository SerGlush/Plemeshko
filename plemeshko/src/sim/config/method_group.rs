use serde::Deserialize;

use crate::env::{config::{Config, ConfigId, ConfigLabel, Serializable}, text::TextId};

use super::method::{MethodId, MethodLabel};

#[derive(Deserialize)]
pub struct RawMethodGroup {
    pub variants: Vec<MethodLabel>,
}

pub struct MethodGroup {
    pub name: TextId,
    pub variants: Vec<MethodId>,
}

pub type MethodGroupLabel = ConfigLabel<MethodGroup>;
pub type MethodGroupId = ConfigId<MethodGroup>;

impl Config for MethodGroup {
    type Raw = RawMethodGroup;

    const TAG: &'static str = "method-group";

    fn prepare(
        raw: Self::Raw,
        label: ConfigLabel<Self>,
        indexer: &mut crate::env::config::ConfigIndexer,
    ) -> Self {
        Self {
            name: config_text_id!(label),
            variants: Serializable::from_serializable(raw.variants, indexer),
        }
    }
}
