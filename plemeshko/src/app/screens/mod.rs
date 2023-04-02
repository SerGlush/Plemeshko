use anyhow::Result;
use egui::CentralPanel;
use enum_map::{enum_map, Enum};

use crate::state::AppState;

use super::{
    env::Env,
    events::{FlagEvent, SetEvent},
    widgets::{ScreenTransitionEvent, Screens, Widget},
};

mod load;
mod main;
mod menu;

#[derive(Enum, Clone, Copy, Default)]
pub enum AppScreen {
    #[default]
    Menu,
    Load,
    Main,
}

pub struct App(Screens<(), AppScreen, App>, Env<'static>);

pub type AppScreenTransitionEvent = ScreenTransitionEvent<AppScreen, App>;

pub struct AppLoadEvent(SetEvent<String>);
pub struct AppExitEvent(FlagEvent);

impl App {
    pub fn new() -> Self {
        App(
            Screens::new(enum_map! {
                AppScreen::Load => Box::new(load::LoadScreen::new()) as Box<dyn Widget<Response = ()>>,
                AppScreen::Menu => Box::new(menu::MenuScreen::new()) as Box<dyn Widget<Response = ()>>,
                AppScreen::Main => Box::new(main::MainScreen::new()) as Box<dyn Widget<Response = ()>>,
            }),
            Env::new(),
        )
    }

    pub fn update(&mut self, _st: &mut AppState) -> Result<()> {
        Ok(())
    }

    pub fn ui(&mut self, st: &mut AppState, egui_ctx: &egui::Context) -> Result<bool> {
        let mut ev_load = AppLoadEvent::new();
        let ev_exit = AppExitEvent::new();
        self.1.with(st, |env| {
            env.with(&egui_ctx.clone(), |env| {
                env.with(&ev_load, |env| {
                    env.with(&ev_exit, |env| {
                        CentralPanel::default()
                            .show(egui_ctx, |ui| self.0.ui(env, ui))
                            .inner
                    })
                })
            })
        })?;
        if let Some(save_name) = ev_load.get_mut() {
            crate::state::save::load(save_name, st)?;
        }
        Ok(ev_exit.get())
    }
}

impl AppLoadEvent {
    delegate::delegate! {
        to self.0 {
            pub fn emit(&self, name: String);
            pub fn get_mut(&mut self) -> Option<&mut String>;
        }
    }

    pub fn new() -> Self {
        AppLoadEvent(SetEvent::new())
    }
}

impl AppExitEvent {
    delegate::delegate! {
        to self.0 {
            pub fn emit(&self);
            pub fn get(&self) -> bool;
        }
    }

    pub fn new() -> Self {
        AppExitEvent(FlagEvent::new())
    }
}
