use std::{
    borrow::{Borrow, Cow},
    sync::MutexGuard,
};

use anyhow::{anyhow, Result};
use egui_extras::RetainedImage;
use fluent::FluentArgs;

use crate::{
    sim::Sim,
    state::{
        components::{AppComponents, ComponentId, SharedComponents},
        config::{Config, FatConfigId},
        text::{FatTextId, TextIdRef},
    },
};

use super::{texture::FatTextureId, AppState};

pub trait HasSharedComponents {
    fn shared_components(&self) -> &SharedComponents;
}

pub trait HasAppComponents {
    fn app_components(&self) -> &AppComponents;
}

pub trait HasAppState {
    fn app_state(&self) -> &AppState;
}

pub trait HasSimMutex {
    fn lock_sim(&self) -> MutexGuard<'_, Option<Sim>>;
}

pub trait HasConfigs {
    fn config<C: Config>(&self, id: FatConfigId<C>) -> Result<&C>;
}

pub trait HasTexts {
    fn text<'a>(&'a self, id: &FatTextId) -> Result<Cow<'a, str>>;
    fn text_core<'a>(&'a self, id: &str) -> Result<Cow<'a, str>>;
    fn text_fmt<'a>(&'a self, id: &FatTextId, args: &'a FluentArgs<'_>) -> Result<Cow<'a, str>>;
    fn text_core_fmt<'a>(&'a self, id: &str, args: &'a FluentArgs<'_>) -> Result<Cow<'a, str>>;
}

pub trait HasTextures {
    fn texture(&self, id: FatTextureId) -> Result<&RetainedImage>;
    fn texture_core(&self, label: &str) -> Result<&RetainedImage>;
}

impl<T: HasSharedComponents + ?Sized> HasConfigs for T {
    default fn config<C: Config>(&self, id: FatConfigId<C>) -> Result<&C> {
        self.shared_components().config(id)
    }
}

impl<T: HasAppState + ?Sized> HasAppComponents for T {
    default fn app_components(&self) -> &AppComponents {
        &self.app_state().components
    }
}

impl<T: HasAppComponents + ?Sized> HasTexts for T {
    fn text<'a>(&'a self, id: &FatTextId) -> Result<Cow<'a, str>> {
        self.app_components()
            .component(id.0)?
            .text(id.1.borrow(), None)
    }

    fn text_core<'a>(&'a self, id: &str) -> Result<Cow<'a, str>> {
        self.app_components()
            .component(ComponentId::core())?
            .text(TextIdRef::from_str(id), None)
    }

    fn text_fmt<'a>(&'a self, id: &FatTextId, args: &'a FluentArgs<'_>) -> Result<Cow<'a, str>> {
        self.app_components()
            .component(id.0)?
            .text(id.1.borrow(), Some(args))
    }

    fn text_core_fmt<'a>(&'a self, id: &str, args: &'a FluentArgs<'_>) -> Result<Cow<'a, str>> {
        self.app_components()
            .component(ComponentId::core())?
            .text(TextIdRef::from_str(id), Some(args))
    }
}

impl<T: HasAppState + ?Sized> HasTextures for T {
    fn texture(&self, id: FatTextureId) -> Result<&RetainedImage> {
        let app_st = self.app_state();
        Ok(app_st
            .components
            .component(id.0)?
            .textures
            .get(id.1)
            .unwrap_or(&app_st.fallback_texture))
    }

    fn texture_core(&self, label: &str) -> Result<&RetainedImage> {
        let app_st = self.app_state();
        let core_textures = &app_st.components.component(ComponentId::core())?.textures;
        let id = match core_textures.id_from_raw(label) {
            Ok(id) => id,
            Err(e) => {
                log::warn!("Core texture retrieval failed: {e}");
                return Ok(&app_st.fallback_texture);
            }
        };
        core_textures
            .get(id)
            .ok_or_else(|| anyhow!("Invalid associated label's id: {label}"))
    }
}

impl HasAppState for AppState {
    fn app_state(&self) -> &AppState {
        self
    }
}

impl<T: HasAppState + ?Sized> HasSimMutex for T {
    default fn lock_sim(&self) -> std::sync::MutexGuard<'_, Option<Sim>> {
        self.app_state().shared.sim.lock().unwrap()
    }
}
