use std::borrow::Cow;

use anyhow::{Ok, Result};
use egui::{Image, ImageButton, Response, RichText, Ui, Vec2};

use crate::{
    sim::config::resource::{ResourceIo, ResourceMap},
    state::{
        components::SharedComponents,
        config::{Config, FatConfigId, Info},
        has::HasTexts,
        texture::FatTexturePartId,
        AppState,
    },
};

/// Consume iterator calling a fallible ui callback on each item.
/// Prepends each item's id with its index in the iterator.
pub fn draw_iter_indexed<T>(
    ui: &mut Ui,
    iter: impl Iterator<Item = T>,
    mut add_item_contents: impl FnMut(&mut Ui, T) -> Result<()>,
) -> Result<()> {
    for (index, item) in iter.enumerate() {
        ui.push_id(index, |ui| add_item_contents(ui, item)).inner?;
    }
    Ok(())
}

pub trait ConfigIteratorExt<'a, I, C: 'a>: Sized {
    fn configs(self, shared_comps: &'a SharedComponents) -> Box<dyn Iterator<Item = &'a C> + 'a>;
    fn configs_with_ids(
        self,
        shared_comps: &'a SharedComponents,
    ) -> Box<dyn Iterator<Item = (FatConfigId<C>, &'a C)> + 'a>;
}

impl<'a, C: Config, T: Iterator<Item = FatConfigId<C>> + 'a>
    ConfigIteratorExt<'a, FatConfigId<C>, C> for T
{
    fn configs(self, shared_comps: &'a SharedComponents) -> Box<dyn Iterator<Item = &'a C> + 'a> {
        Box::new(self.map(|id| shared_comps.config(id).unwrap()))
    }

    fn configs_with_ids(
        self,
        shared_comps: &'a SharedComponents,
    ) -> Box<dyn Iterator<Item = (FatConfigId<C>, &'a C)> + 'a> {
        Box::new(self.map(|id| (id, shared_comps.config(id).unwrap())))
    }
}

impl<'a, C: Config, T: Iterator<Item = &'a FatConfigId<C>> + 'a>
    ConfigIteratorExt<'a, &FatConfigId<C>, C> for T
{
    fn configs(self, shared_comps: &'a SharedComponents) -> Box<dyn Iterator<Item = &C> + 'a> {
        Box::new(self.map(|&id| shared_comps.config(id).unwrap()))
    }

    fn configs_with_ids(
        self,
        shared_comps: &'a SharedComponents,
    ) -> Box<dyn Iterator<Item = (FatConfigId<C>, &'a C)> + 'a> {
        Box::new(self.map(|&id| (id, shared_comps.config(id).unwrap())))
    }
}

impl<'a, C: Config> ConfigIteratorExt<'a, Vec<()>, C> for &'a Vec<FatConfigId<C>> {
    fn configs(self, shared_comps: &'a SharedComponents) -> Box<dyn Iterator<Item = &'a C> + 'a> {
        self.iter().configs(shared_comps)
    }

    fn configs_with_ids(
        self,
        shared_comps: &'a SharedComponents,
    ) -> Box<dyn Iterator<Item = (FatConfigId<C>, &'a C)> + 'a> {
        self.iter().configs_with_ids(shared_comps)
    }
}

#[derive(Clone, Copy)]
pub enum Modifier {
    None,
    Command,
    Shift,
}

impl Modifier {
    pub fn elim<R>(self, on_none: R, on_shift: R, on_cmd: R) -> R {
        match self {
            Modifier::None => on_none,
            Modifier::Command => on_cmd,
            Modifier::Shift => on_shift,
        }
    }
}

pub fn using_modifiers<R>(response: &egui::Response, f: impl FnOnce(Modifier) -> R) -> R {
    let mods = &response.ctx.input().modifiers;
    if mods.shift_only() {
        f(Modifier::Shift)
    } else if mods.command_only() {
        f(Modifier::Command)
    } else {
        f(Modifier::None)
    }
}

pub fn on_using_modifiers<R>(
    response: &egui::Response,
    on: impl FnOnce(&egui::Response) -> bool,
    f: impl FnOnce(Modifier) -> R,
) -> Option<R> {
    if on(response) {
        Some(using_modifiers(response, f))
    } else {
        None
    }
}

pub fn draw_icon(
    app_st: &AppState,
    ctx: &egui::Context,
    ui: &mut Ui,
    icon: &FatTexturePartId,
    siz: Vec2,
    sty: impl FnOnce(Image) -> Image,
) -> Result<Response> {
    let mut button = Image::new(app_st.texture(icon.texture)?.texture_id(ctx), siz);
    if let Some(uv) = icon.uv {
        button = button.uv(uv);
    }
    Ok(ui.add(sty(button)))
}

pub fn draw_icon_with_tooltip(
    app_st: &AppState,
    ctx: &egui::Context,
    ui: &mut Ui,
    info: &Info,
    siz: Vec2,
    sty: impl FnOnce(Image) -> Image,
    ex_ui: impl FnOnce(&mut Ui),
) -> Result<()> {
    let mut button = Image::new(app_st.texture(info.icon.texture)?.texture_id(ctx), siz);
    if let Some(uv) = info.icon.uv {
        button = button.uv(uv);
    }
    let response = ui.add(sty(button));
    let name = app_st.text(&info.name)?;
    let description = app_st.text(&info.description)?;
    response.on_hover_ui_at_pointer(|ui| {
        ui.strong(name);
        ui.label(description);
        ex_ui(ui);
    });
    Ok(())
}

pub fn draw_icon_btn_with_tooltip(
    app_st: &AppState,
    ctx: &egui::Context,
    ui: &mut Ui,
    info: &Info,
    siz: Vec2,
    sty: impl FnOnce(ImageButton) -> ImageButton,
    ex_ui: impl FnOnce(&mut Ui),
    click: impl FnOnce() -> Result<()>,
) -> Result<()> {
    let mut button = ImageButton::new(app_st.texture(info.icon.texture)?.texture_id(ctx), siz);
    if let Some(uv) = info.icon.uv {
        button = button.uv(uv);
    }
    let response = ui.add(sty(button));
    let name = app_st.text(&info.name)?;
    let description = app_st.text(&info.description)?;
    let response = response.on_hover_ui_at_pointer(|ui| {
        ui.strong(name);
        ui.label(description);
        ex_ui(ui);
    });
    if response.clicked() {
        click()?;
    }
    Ok(())
}

pub fn draw_resource_map(
    app_st: &AppState,
    shared_comps: &SharedComponents,
    ctx: &egui::Context,
    ui: &mut Ui,
    rm: &ResourceMap,
) -> Result<()> {
    for (&id, &amount) in rm {
        ui.horizontal(|ui| {
            draw_icon(
                app_st,
                ctx,
                ui,
                &shared_comps.config(id)?.info.icon,
                egui::vec2(24., 24.),
                |i| i,
            )?;
            ui.label(amount.to_string());
            Ok(())
        })
        .inner?;
    }
    Ok(())
}

pub fn draw_resource_map_labeled(
    app_st: &AppState,
    shared_comps: &SharedComponents,
    ctx: &egui::Context,
    ui: &mut Ui,
    rm: &ResourceMap,
    label: impl Into<RichText>,
    skip_empty: bool,
) -> Result<()> {
    if skip_empty && rm.is_empty() {
        return Ok(());
    }
    ui.strong(label);
    ui.indent("output", |ui| {
        draw_resource_map(app_st, shared_comps, ctx, ui, rm)
    })
    .inner
}

pub fn draw_resource_io_lazy<'a>(
    app_st: &AppState,
    shared_comps: &SharedComponents,
    ctx: &egui::Context,
    ui: &mut Ui,
    rio: impl FnOnce() -> Cow<'a, ResourceIo> + 'a,
) -> Result<()> {
    let rio = rio();
    draw_resource_map_labeled(
        app_st,
        shared_comps,
        ctx,
        ui,
        &rio.input,
        app_st.text_core("ui_generic_input")?,
        true,
    )?;
    draw_resource_map_labeled(
        app_st,
        shared_comps,
        ctx,
        ui,
        &rio.output,
        app_st.text_core("ui_generic_output")?,
        true,
    )?;
    Ok(())
}

pub fn draw_resource_io_tt_lazy<'a>(
    app_st: &AppState,
    shared_comps: &SharedComponents,
    ctx: &egui::Context,
    response: egui::Response,
    rio: impl FnOnce() -> Cow<'a, ResourceIo> + 'a,
) -> egui::Response {
    response.on_hover_ui(|ui| {
        draw_resource_io_lazy(app_st, shared_comps, ctx, ui, rio).unwrap();
    })
}

pub fn draw_resource_io(
    app_st: &AppState,
    shared_comps: &SharedComponents,
    ctx: &egui::Context,
    ui: &mut Ui,
    rio: &ResourceIo,
) -> Result<()> {
    draw_resource_io_lazy(app_st, shared_comps, ctx, ui, || Cow::Borrowed(rio))
}

pub fn draw_resource_io_tt(
    app_st: &AppState,
    shared_comps: &SharedComponents,
    ctx: &egui::Context,
    response: egui::Response,
    rio: &ResourceIo,
) -> egui::Response {
    draw_resource_io_tt_lazy(app_st, shared_comps, ctx, response, || Cow::Borrowed(rio))
}
