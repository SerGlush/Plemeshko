use serde::Deserialize;

use crate::{
    sim::units::ResourceWeight,
    state::config::{Config, FatConfigId, FatConfigLabel, Info, Prepare, RawInfo},
};

use super::{
    resource::{RawResourceIo, ResourceIo},
    transport_group::{TransportGroup, TransportGroupId},
};

#[derive(Deserialize)]
pub struct RawTransportMethod {
    #[serde(flatten)]
    pub info: RawInfo,
    pub group: FatConfigLabel<TransportGroup>,
    pub capacity: ResourceWeight,
    pub fuel: RawResourceIo,
    pub ui_priority: u32,
    #[serde(default)]
    pub initially_unlocked: bool,
}

#[derive(Debug)]
pub struct TransportMethod {
    pub info: Info,
    pub group: TransportGroupId,
    pub capacity: ResourceWeight,
    pub fuel: ResourceIo,
    pub ui_priority: u32,
    pub initially_unlocked: bool,
}

pub type TransportMethodId = FatConfigId<TransportMethod>;

impl Prepare for RawTransportMethod {
    type Prepared = TransportMethod;

    fn prepare(
        self,
        ctx: &mut crate::state::config::ConfigsLoadingContext<'_>,
        tif: &mut crate::state::text::TextIdFactory,
    ) -> anyhow::Result<Self::Prepared> {
        let info = self.info.prepare(ctx, tif)?;
        tif.with_lock(|tif| {
            Ok(TransportMethod {
                info,
                group: self.group.prepare(ctx, tif)?,
                capacity: self.capacity,
                fuel: self.fuel.prepare(ctx, tif)?,
                ui_priority: self.ui_priority,
                initially_unlocked: self.initially_unlocked,
            })
        })
    }
}

impl Config for TransportMethod {
    type Raw = RawTransportMethod;

    const TAG: &'static str = "transport-method";
}
