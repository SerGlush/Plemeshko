use std::path::PathBuf;

use anyhow::{Context, Result};
use egui::CentralPanel;
use enum_map::{enum_map, Enum};
use unic_langid::LanguageIdentifier;

use crate::state::{components::ComponentId, AppState};

use super::{
    env::Env,
    events::{FlagEvent, SetEvent},
    widgets::{ScreenTransitionEvent, Screens, Widget},
};

mod load;
mod main;
mod menu;
mod new_game;

#[derive(Enum, Clone, Copy, Default)]
pub enum AppScreen {
    #[default]
    Menu,
    NewGame,
    Load,
    Main,
}

pub struct App(Screens<(), AppScreen, App>, Env<'static>);

pub type AppScreenTransitionEvent = ScreenTransitionEvent<AppScreen, App>;

pub struct AppSaveEvent(FlagEvent);
pub struct AppLoadEvent(SetEvent<String>);
pub struct AppExitEvent(FlagEvent);
pub struct AppNewGameEvent(SetEvent<String>);
pub struct AppSwitchTranslationEvent(SetEvent<(LanguageIdentifier, PathBuf)>);

impl App {
    pub fn new() -> Self {
        App(
            Screens::new(enum_map! {
                AppScreen::Menu => Box::new(menu::MenuScreen::new()) as Box<dyn Widget<Response = ()>>,
                AppScreen::NewGame => Box::new(new_game::NewGameScreen::new()) as Box<dyn Widget<Response = ()>>,
                AppScreen::Load => Box::new(load::LoadScreen::new()) as Box<dyn Widget<Response = ()>>,
                AppScreen::Main => Box::new(main::MainScreen::new()) as Box<dyn Widget<Response = ()>>,
            }),
            Env::new(),
        )
    }

    pub fn update(&mut self, _st: &mut AppState) -> Result<()> {
        Ok(())
    }

    pub fn ui(&mut self, st: &mut AppState, egui_ctx: &egui::Context) -> Result<bool> {
        let ev_save = AppSaveEvent(FlagEvent::new());
        let mut ev_load = AppLoadEvent(SetEvent::new());
        let ev_exit = AppExitEvent(FlagEvent::new());
        let mut ev_newgame = AppNewGameEvent(SetEvent::new());
        let mut ev_sw_translation = AppSwitchTranslationEvent(SetEvent::new());
        self.1.with(st, |env| {
            env.with(&egui_ctx.clone(), |env| {
                env.with(&ev_save, |env| {
                    env.with(&ev_load, |env| {
                        env.with(&ev_exit, |env| {
                            env.with(&ev_newgame, |env| {
                                env.with(&ev_sw_translation, |env| {
                                    CentralPanel::default()
                                        .show(egui_ctx, |ui| self.0.ui(env, ui))
                                        .inner
                                })
                            })
                        })
                    })
                })
            })
        })?;
        if ev_save.0.get() {
            crate::state::save::save(st)?;
        }
        if let Some(save_name) = ev_load.0.get_mut() {
            crate::state::save::load(save_name, st)?;
        }
        if let Some(game_name) = ev_newgame.0.get_mut() {
            let mut sim_guard = st.shared.sim.lock().unwrap();
            *sim_guard = Some(
                crate::sim::Sim::new(&mut st.shared.components.write().unwrap())
                    .context("Creating new game")?,
            );
            st.session = Some(game_name.clone());
        }
        if let Some(tr) = ev_sw_translation.0.get_mut() {
            for (id, c) in st.components.iter_components_mut() {
                if id == ComponentId::core() {
                    c.texts
                        .switch_translation_exact(tr.0.clone(), tr.1.clone())?;
                    continue;
                }
                c.texts.switch_translation(tr.0.clone())?;
            }
        }
        Ok(ev_exit.0.get())
    }
}

impl AppSaveEvent {
    delegate::delegate! {
        to self.0 {
            pub fn emit(&self);
        }
    }
}

impl AppLoadEvent {
    delegate::delegate! {
        to self.0 {
            pub fn emit(&self, name: String);
        }
    }
}

impl AppExitEvent {
    delegate::delegate! {
        to self.0 {
            pub fn emit(&self);
        }
    }
}

impl AppNewGameEvent {
    delegate::delegate! {
        to self.0 {
            pub fn emit(&self, name: String);
        }
    }
}

impl AppSwitchTranslationEvent {
    delegate::delegate! {
        to self.0 {
            pub fn emit(&self, tr: (LanguageIdentifier, PathBuf));
        }
    }
}
