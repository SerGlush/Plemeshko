use serde::Deserialize;

use crate::state::{
    config::{Config, FatConfigId, Prepare},
    text::FatTextId,
};

#[derive(Deserialize)]
pub struct RawTransportGroup {}

pub struct TransportGroup {
    pub name: FatTextId,
}

pub type TransportGroupId = FatConfigId<TransportGroup>;

impl Prepare for RawTransportGroup {
    type Prepared = TransportGroup;

    fn prepare(
        self,
        ctx: &mut crate::state::config::ConfigsLoadingContext<'_>,
        tif: &mut crate::state::text::TextIdFactory,
    ) -> anyhow::Result<Self::Prepared> {
        let name = tif.create("name").in_component(ctx.component_id());
        Ok(TransportGroup { name })
    }
}

impl Config for TransportGroup {
    type Raw = RawTransportGroup;

    const TAG: &'static str = "transport-group";
}
