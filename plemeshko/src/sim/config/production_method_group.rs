use anyhow::Result;
use serde::Deserialize;

use crate::state::{
    config::{Config, ConfigsLoadingContext, FatConfigId, FatConfigLabel, Prepare},
    text::{FatTextId, TextIdFactory},
};

use super::production_method::{ProductionMethod, ProductionMethodId};

#[derive(Deserialize)]
pub struct RawProductionMethodGroup {
    pub variants: Vec<FatConfigLabel<ProductionMethod>>,
}

#[derive(Debug)]
pub struct ProductionMethodGroup {
    pub name: FatTextId,
    pub variants: Vec<ProductionMethodId>,
}

pub type ProductionMethodGroupId = FatConfigId<ProductionMethodGroup>;

impl Prepare for RawProductionMethodGroup {
    type Prepared = ProductionMethodGroup;

    fn prepare(
        self,
        ctx: &mut ConfigsLoadingContext<'_>,
        tif: &mut TextIdFactory,
    ) -> Result<ProductionMethodGroup> {
        let name = tif.create("name").in_component(ctx.this_component.id());
        tif.with_lock(|tif| {
            Ok(ProductionMethodGroup {
                name,
                variants: self.variants.prepare(ctx, tif)?,
            })
        })
    }
}

impl Config for ProductionMethodGroup {
    type Raw = RawProductionMethodGroup;

    const TAG: &'static str = "production-method-group";
}
