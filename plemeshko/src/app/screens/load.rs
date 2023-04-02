use anyhow::{Ok, Result};
use egui::vec2;

use crate::app::{
    env::Env,
    events::FlagEvent,
    widgets::{Menu, ScaledMenuItemBlank, ScaledMenuItemButton, Widget},
};

use super::{AppLoadEvent, AppScreen, AppScreenTransitionEvent};

struct LoadScreenId;

pub struct LoadScreen {
    fetched: bool,
    menu: Menu<LoadScreenId>,
}

struct LoadScreenRefresh(FlagEvent);

const MENU_BUTTON_HEIGHT: f32 = 0.1;
const MENU_FONT_SIZE_BASE: f32 = 10.;
const MENU_FONT_SCALE: f32 = 0.02;
const MENU_ITEM_MARGIN: f32 = 0.02;
const MENU_WIDTH: f32 = 0.2;

impl LoadScreen {
    pub fn new() -> Self {
        LoadScreen {
            fetched: false,
            menu: Menu::new(egui::Layout::top_down(egui::Align::Center), Vec::new()),
        }
    }
}

impl Widget for LoadScreen {
    type Response = ();

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut egui::Ui) -> Result<Self::Response> {
        if !self.fetched {
            let saves = crate::state::save::saves()?;
            if saves.len() == 1 {
                env.get::<AppLoadEvent>().unwrap().emit(saves[0].0.clone());
                env.get::<AppScreenTransitionEvent>()
                    .unwrap()
                    .emit(AppScreen::Main);
                return Ok(());
            }
            self.fetched = true;
            self.menu.items.reserve(saves.len());
            for (name, save) in saves {
                self.menu
                    .items
                    .push(Box::new(ScaledMenuItemButton::<_, _, LoadScreenId>::new(
                        vec2(MENU_WIDTH, 1.5 * MENU_BUTTON_HEIGHT),
                        Menu::<LoadScreenId>::simple_scaled_text(
                            format!(
                                "Load \"{name}\"\n{}\ntime played: {}",
                                save.saved_date,
                                save.play_time.as_secs()
                            ),
                            MENU_FONT_SIZE_BASE * 0.75,
                            MENU_FONT_SCALE,
                        ),
                        move |env| {
                            env.get::<AppLoadEvent>().unwrap().emit(name.clone());
                            env.get::<AppScreenTransitionEvent>()
                                .unwrap()
                                .emit(AppScreen::Main);
                            env.get::<LoadScreenRefresh>().unwrap().0.emit();
                            Ok(())
                        },
                    )));
                self.menu
                    .items
                    .push(Box::new(ScaledMenuItemBlank::<LoadScreenId>::new(vec2(
                        MENU_WIDTH,
                        MENU_ITEM_MARGIN,
                    ))));
            }
            self.menu
                .items
                .push(Box::new(ScaledMenuItemButton::<_, _, LoadScreenId>::new(
                    vec2(MENU_WIDTH, MENU_BUTTON_HEIGHT),
                    Menu::<LoadScreenId>::simple_scaled_text(
                        "Return",
                        MENU_FONT_SIZE_BASE,
                        MENU_FONT_SCALE,
                    ),
                    |env| {
                        env.get::<AppScreenTransitionEvent>()
                            .unwrap()
                            .emit(AppScreen::Menu);
                        env.get::<LoadScreenRefresh>().unwrap().0.emit();
                        Ok(())
                    },
                )));
        }
        let ev_refresh = LoadScreenRefresh(FlagEvent::new());
        env.with(&ev_refresh, |env| self.menu.ui(env, ui))?;
        if ev_refresh.0.get() {
            self.fetched = false;
            self.menu.items.clear();
        }
        Ok(())
    }
}
