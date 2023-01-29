use serde::Deserialize;

use crate::{
    sim::units::ResourceWeight,
    state::{
        config::{Config, FatConfigId, FatConfigLabel, Prepare},
        text::FatTextId,
    },
};

use super::{
    resource::{RawResourceIo, ResourceIo},
    transport_group::{TransportGroup, TransportGroupId},
};

#[derive(Deserialize)]
pub struct RawTransport {
    pub group: FatConfigLabel<TransportGroup>,
    pub capacity: ResourceWeight,
    pub fuel: RawResourceIo,
}

pub struct Transport {
    pub name: FatTextId,
    pub group: TransportGroupId,
    pub capacity: ResourceWeight,
    pub fuel: ResourceIo,
}

pub type TransportId = FatConfigId<Transport>;

impl Prepare for RawTransport {
    type Prepared = Transport;

    fn prepare(
        self,
        ctx: &mut crate::state::config::ConfigsLoadingContext<'_>,
        tif: &mut crate::state::text::TextIdFactory,
    ) -> anyhow::Result<Self::Prepared> {
        let name = tif.create("name").in_component(ctx.this_component.id());
        tif.with_lock(|tif| {
            Ok(Transport {
                name,
                group: self.group.prepare(ctx, tif)?,
                capacity: self.capacity,
                fuel: self.fuel.prepare(ctx, tif)?,
            })
        })
    }
}

impl Config for Transport {
    type Raw = RawTransport;

    const TAG: &'static str = "transport";
}
