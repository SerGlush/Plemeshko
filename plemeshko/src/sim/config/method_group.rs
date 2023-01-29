use anyhow::Result;
use serde::Deserialize;

use crate::state::{
    config::{Config, ConfigsLoadingContext, FatConfigId, FatConfigLabel, Prepare},
    text::{FatTextId, TextIdFactory},
};

use super::method::{Method, MethodId};

#[derive(Deserialize)]
pub struct RawMethodGroup {
    pub variants: Vec<FatConfigLabel<Method>>,
}

pub struct MethodGroup {
    pub name: FatTextId,
    pub variants: Vec<MethodId>,
}

pub type MethodGroupId = FatConfigId<MethodGroup>;

impl Prepare for RawMethodGroup {
    type Prepared = MethodGroup;

    fn prepare(
        self,
        ctx: &mut ConfigsLoadingContext<'_>,
        tif: &mut TextIdFactory,
    ) -> Result<MethodGroup> {
        let name = tif.create("name").in_component(ctx.this_component.id());
        tif.with_lock(|tif| {
            Ok(MethodGroup {
                name,
                variants: self.variants.prepare(ctx, tif)?,
            })
        })
    }
}

impl Config for MethodGroup {
    type Raw = RawMethodGroup;

    const TAG: &'static str = "method-group";
}
