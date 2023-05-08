use serde::Deserialize;

use crate::state::{
    text::FatTextId,
    texture::{FatTextureId, FatTexturePartId, FatTexturePartLabel},
};

use super::Prepare;

/// Deserialized part of [`Info`]
#[derive(Deserialize)]
pub struct RawInfo {
    #[serde(flatten)]
    pub icon: Option<FatTexturePartLabel>,
}

/// User-facing information about some config
#[derive(Debug)]
pub struct Info {
    pub name: FatTextId,
    pub description: FatTextId,
    pub icon: FatTexturePartId,
}

impl Prepare for RawInfo {
    type Prepared = Info;

    fn prepare(
        self,
        ctx: &mut super::ConfigsLoadingContext<'_>,
        tif: &mut crate::state::text::TextIdFactory,
    ) -> anyhow::Result<Self::Prepared> {
        let name = tif.create("name").in_component(ctx.this_component.id());
        let description = tif
            .create("description")
            .in_component(ctx.this_component.id());
        Ok(Info {
            name,
            description,
            icon: match self.icon {
                Some(icon) => tif.with_lock(|tif| icon.prepare(ctx, tif))?,
                None => FatTextureId::new_invalid().to_part(),
            },
        })
    }
}
