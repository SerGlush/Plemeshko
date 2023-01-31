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
pub struct RawTransportMethod {
    pub group: FatConfigLabel<TransportGroup>,
    pub capacity: ResourceWeight,
    pub fuel: RawResourceIo,
    pub ui_priority: u32,
}

pub struct TransportMethod {
    pub name: FatTextId,
    pub group: TransportGroupId,
    pub capacity: ResourceWeight,
    pub fuel: ResourceIo,
    pub ui_priority: u32,
}

pub type TransportMethodId = FatConfigId<TransportMethod>;

impl Prepare for RawTransportMethod {
    type Prepared = TransportMethod;

    fn prepare(
        self,
        ctx: &mut crate::state::config::ConfigsLoadingContext<'_>,
        tif: &mut crate::state::text::TextIdFactory,
    ) -> anyhow::Result<Self::Prepared> {
        let name = tif.create("name").in_component(ctx.this_component.id());
        tif.with_lock(|tif| {
            Ok(TransportMethod {
                name,
                group: self.group.prepare(ctx, tif)?,
                capacity: self.capacity,
                fuel: self.fuel.prepare(ctx, tif)?,
                ui_priority: self.ui_priority,
            })
        })
    }
}

impl Config for TransportMethod {
    type Raw = RawTransportMethod;

    const TAG: &'static str = "transport-method";
}
