use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    state::{
        components::{ComponentId, RawFatLabel},
        config::Prepare,
        AppState,
    },
    util::Rect,
};

pub(super) type RawTextureId = u32;

#[derive(Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct TextureLabel(pub(super) String);

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct FatTextureLabel(RawFatLabel);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureId(pub(super) RawTextureId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FatTextureId(pub ComponentId, pub TextureId);

#[derive(Serialize, Deserialize)]
pub struct FatTexturePartLabel {
    pub texture: FatTextureLabel,
    #[serde(flatten)]
    pub uv: Option<Rect<f32>>,
}

pub struct FatTexturePartId {
    pub texture: FatTextureId,
    pub uv: Option<Rect<f32>>,
}

impl FatTexturePartId {
    pub fn draw(
        &self,
        app_st: &AppState,
        egui_ctx: &egui::Context,
        size: impl Into<egui::Vec2>,
    ) -> Result<egui::Image> {
        let texture = app_st.get_texture(self.texture)?;
        let egui_texture_id = texture.texture_id(egui_ctx);
        let widget = egui::Image::new(egui_texture_id, size);
        Ok(match self.uv {
            Some(uv) => widget.uv(uv),
            None => widget,
        })
    }
}

impl Prepare for TextureLabel {
    type Prepared = TextureId;

    fn prepare(
        self,
        ctx: &mut crate::state::config::ConfigsLoadingContext<'_>,
        _tif: &mut crate::state::text::TextIdFactory,
    ) -> Result<Self::Prepared> {
        ctx.this_component.textures.get_id(&self)
    }
}

impl Prepare for FatTextureLabel {
    type Prepared = FatTextureId;

    fn prepare(
        self,
        ctx: &mut crate::state::config::ConfigsLoadingContext<'_>,
        _tif: &mut crate::state::text::TextIdFactory,
    ) -> Result<Self::Prepared> {
        Ok(match &self.0 .0 {
            Some(comp_label) => {
                let comp_id = ctx.other_components.indexer.get_id(comp_label)?;
                let tex_id = ctx
                    .other_components
                    .app
                    .get_component(comp_id)?
                    .textures
                    .get_id_from_raw(&self.0 .1)?;
                FatTextureId(comp_id, tex_id)
            }
            None => {
                let tex_id = ctx.this_component.textures.get_id_from_raw(&self.0 .1)?;
                FatTextureId(ctx.this_component.id(), tex_id)
            }
        })
    }
}

impl Prepare for FatTexturePartLabel {
    type Prepared = FatTexturePartId;

    fn prepare(
        self,
        ctx: &mut crate::state::config::ConfigsLoadingContext<'_>,
        tif: &mut crate::state::text::TextIdFactory,
    ) -> Result<Self::Prepared> {
        let texture = self.texture.prepare(ctx, tif)?;
        Ok(FatTexturePartId {
            texture,
            uv: self.uv,
        })
    }
}
