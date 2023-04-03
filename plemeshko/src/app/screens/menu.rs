use anyhow::{Ok, Result};
use egui::vec2;

use crate::{
    app::{
        env::Env,
        widgets::{Menu, ScaledMenuItemBlank, ScaledMenuItemButton, Widget},
    },
    state::AppState, params::{MENU_ITEM_SIZE_1, MENU_ITEM_SIZE_2, MENU_FONT_SIZE_BASE, MENU_FONT_SCALE, MENU_ITEM_MARGIN},
};

use super::{AppExitEvent, AppScreenTransitionEvent};

struct MenuScreenId;

pub struct MenuScreen(Menu<MenuScreenId>);

impl MenuScreen {
    pub fn new() -> Self {
        MenuScreen(Menu::new(
            egui::Layout::top_down(egui::Align::Center),
            vec![
                Box::new(ScaledMenuItemButton::<_, _, MenuScreenId>::new(
                    vec2(MENU_ITEM_SIZE_2, MENU_ITEM_SIZE_1),
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
                    MENU_ITEM_SIZE_2,
                    MENU_ITEM_MARGIN,
                ))),
                Box::new(ScaledMenuItemButton::<_, _, MenuScreenId>::new(
                    vec2(MENU_ITEM_SIZE_2, MENU_ITEM_SIZE_1),
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
                    MENU_ITEM_SIZE_2,
                    MENU_ITEM_MARGIN,
                ))),
                Box::new(ScaledMenuItemButton::<_, _, MenuScreenId>::new(
                    vec2(MENU_ITEM_SIZE_2, MENU_ITEM_SIZE_1),
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
