use anyhow::{Ok, Result};
use egui::vec2;

use crate::{
    app::{
        env::Env,
        widgets::{Menu, ScaledMenuItemBlank, ScaledMenuItemButton, Widget},
    },
    state::AppState,
};

use super::{AppExitEvent, AppScreenTransitionEvent};

struct MenuScreenId;

pub struct MenuScreen(Menu<MenuScreenId>);

const MENU_BUTTON_HEIGHT: f32 = 0.1;
const MENU_FONT_SIZE_BASE: f32 = 10.;
const MENU_FONT_SCALE: f32 = 0.02;
const MENU_ITEM_MARGIN: f32 = 0.02;
const MENU_WIDTH: f32 = 0.2;

impl MenuScreen {
    pub fn new() -> Self {
        MenuScreen(Menu::new(
            egui::Layout::top_down(egui::Align::Center),
            vec![
                Box::new(ScaledMenuItemButton::<_, _, MenuScreenId>::new(
                    vec2(MENU_WIDTH, MENU_BUTTON_HEIGHT),
                    Menu::<MenuScreenId>::simple_scaled_text(
                        "New Game",
                        MENU_FONT_SIZE_BASE,
                        MENU_FONT_SCALE,
                    ),
                    |env: &mut Env<'_>| {
                        let mut sim_guard =
                            env.get::<AppState>().unwrap().shared.sim.lock().unwrap();
                        *sim_guard = Some(crate::sim::Sim::new());
                        env.get::<AppScreenTransitionEvent>()
                            .unwrap()
                            .emit(super::AppScreen::Main);
                        Ok(())
                    },
                )),
                Box::new(ScaledMenuItemBlank::<MenuScreenId>::new(vec2(
                    MENU_WIDTH,
                    MENU_ITEM_MARGIN,
                ))),
                Box::new(ScaledMenuItemButton::<_, _, MenuScreenId>::new(
                    vec2(MENU_WIDTH, MENU_BUTTON_HEIGHT),
                    Menu::<MenuScreenId>::simple_scaled_text(
                        "Continue",
                        MENU_FONT_SIZE_BASE,
                        MENU_FONT_SCALE,
                    ),
                    |env: &mut Env<'_>| {
                        env.get::<AppScreenTransitionEvent>()
                            .unwrap()
                            .emit(super::AppScreen::Load);
                        Ok(())
                    },
                )),
                Box::new(ScaledMenuItemBlank::<MenuScreenId>::new(vec2(
                    MENU_WIDTH,
                    MENU_ITEM_MARGIN,
                ))),
                Box::new(ScaledMenuItemButton::<_, _, MenuScreenId>::new(
                    vec2(MENU_WIDTH, MENU_BUTTON_HEIGHT),
                    Menu::<MenuScreenId>::simple_scaled_text(
                        "Exit",
                        MENU_FONT_SIZE_BASE,
                        MENU_FONT_SCALE,
                    ),
                    |env: &mut Env<'_>| {
                        env.get::<AppExitEvent>().unwrap().emit();
                        Ok(())
                    },
                )),
            ],
        ))
    }
}

impl Widget for MenuScreen {
    type Response = ();

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut egui::Ui) -> Result<Self::Response> {
        self.0.ui(env, ui)
    }
}
