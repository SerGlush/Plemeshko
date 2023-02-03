use anyhow::Result;
use egui::CentralPanel;
use enum_map::{enum_map, Enum};

use crate::{app::screens::main::MainScreen, state::AppState};

use super::{
    env::Env,
    widgets::{Screens, Widget},
};

mod main;

#[derive(Enum, Clone, Copy)]
pub enum AppScreen {
    Main,
}

pub struct App(Screens<(), AppScreen, App>, Env<'static>);

impl App {
    pub fn new() -> Self {
        App(
            Screens::new_at(
                AppScreen::Main,
                enum_map! {
                    AppScreen::Main => Box::new(MainScreen::new()),
                },
            ),
            Env::new(),
        )
    }

    pub fn update(&mut self, _st: &mut AppState) -> Result<()> {
        Ok(())
    }

    pub fn ui(&mut self, st: &mut AppState, egui_ctx: &egui::Context) -> Result<()> {
        self.1.with(st, |env| {
            env.with(&egui_ctx.clone(), |env| {
                CentralPanel::default()
                    .show(egui_ctx, |ui| self.0.ui(env, ui))
                    .inner
            })
        })
    }
}
