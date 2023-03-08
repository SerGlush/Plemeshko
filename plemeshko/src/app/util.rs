use anyhow::Result;
use egui::Ui;

use crate::state::{
    components::SharedComponents,
    config::{Config, FatConfigId},
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
