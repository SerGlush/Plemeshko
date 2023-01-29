use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    sim::units::{ResourceAmount, ResourceWeight},
    state::{
        components::ComponentsRef,
        config::{Config, FatConfigId, FatConfigLabel, Prepare},
        serializable::Serializable,
        text::FatTextId,
    },
};

use super::transport_group::{TransportGroup, TransportGroupId};

#[derive(Deserialize)]
pub struct RawResource {
    pub transport_group: FatConfigLabel<TransportGroup>,
    pub transport_weight: ResourceWeight,
}

#[derive(Debug)]
pub struct Resource {
    pub name: FatTextId,
    pub transport_group: TransportGroupId,
    pub transport_weight: ResourceWeight,
}

pub type ResourceId = FatConfigId<Resource>;

pub type RawResourceMap = HashMap<FatConfigLabel<Resource>, ResourceAmount>;
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

impl Prepare for RawResource {
    type Prepared = Resource;

    fn prepare(
        self,
        ctx: &mut crate::state::config::ConfigsLoadingContext<'_>,
        tif: &mut crate::state::text::TextIdFactory,
    ) -> anyhow::Result<Self::Prepared> {
        let name = tif.create("name").in_component(ctx.this_component.id());
        tif.with_lock(|tif| {
            Ok(Resource {
                name,
                transport_group: self.transport_group.prepare(ctx, tif)?,
                transport_weight: self.transport_weight,
            })
        })
    }
}

impl Config for Resource {
    type Raw = RawResource;

    const TAG: &'static str = "resource";
}

impl Prepare for RawResourceIo {
    type Prepared = ResourceIo;

    fn prepare(
        self,
        ctx: &mut crate::state::config::ConfigsLoadingContext<'_>,
        tif: &mut crate::state::text::TextIdFactory,
    ) -> anyhow::Result<Self::Prepared> {
        Ok(ResourceIo {
            input: tif.with_branch("input", |tif| self.input.prepare(ctx, tif))?,
            output: tif.with_branch("output", |tif| self.output.prepare(ctx, tif))?,
        })
    }
}

impl Serializable for ResourceIo {
    type Raw = RawResourceIo;

    fn from_serializable(raw: RawResourceIo, ctx: ComponentsRef<'_>) -> Result<Self> {
        Ok(ResourceIo {
            input: Serializable::from_serializable(raw.input, ctx)?,
            output: Serializable::from_serializable(raw.output, ctx)?,
        })
    }

    fn into_serializable(self, ctx: ComponentsRef<'_>) -> anyhow::Result<RawResourceIo> {
        Ok(RawResourceIo {
            input: self.input.into_serializable(ctx)?,
            output: self.output.into_serializable(ctx)?,
        })
    }
}
