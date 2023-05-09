use std::path::PathBuf;

use anyhow::{Ok, Result};
use egui::{vec2, Button};
use unic_langid::LanguageIdentifier;

use crate::{
    app::{env::Env, widgets::Widget},
    state::{has::HasTexts, text::TextRepository},
};

use super::{AppExitEvent, AppScreenTransitionEvent, AppSwitchTranslationEvent};

struct MenuScreenId;

pub struct MenuScreen {
    language_select_open: Option<Vec<(LanguageIdentifier, PathBuf)>>,
}

impl MenuScreen {
    pub fn new() -> Self {
        MenuScreen {
            language_select_open: None,
        }
    }
}

impl Widget for MenuScreen {
    type Response = ();

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut egui::Ui) -> Result<Self::Response> {
        let app_st = env.app_state();
        ui.heading(app_st.text_core("ui_menu_title")?);
        ui.indent("menu btns", |ui| {
            let btnsz = vec2(
                (ui.available_width() / 5.).min(200.),
                (ui.available_height() / 10.).min(100.),
            );
            if ui
                .add_sized(btnsz, Button::new(app_st.text_core("ui_menu_new-game")?))
                .clicked()
            {
                env.get::<AppScreenTransitionEvent>()
                    .unwrap()
                    .emit(super::AppScreen::NewGame);
            }
            if ui
                .add_sized(btnsz, Button::new(app_st.text_core("ui_menu_continue")?))
                .clicked()
            {
                env.get::<AppScreenTransitionEvent>()
                    .unwrap()
                    .emit(super::AppScreen::Load);
            }
            if ui
                .add_sized(btnsz, Button::new(app_st.text_core("ui_menu_language")?))
                .clicked()
            {
                self.language_select_open = Some(TextRepository::available_translations_core()?);
            }
            if ui
                .add_sized(btnsz, Button::new(app_st.text_core("ui_menu_exit")?))
                .clicked()
            {
                env.get::<AppExitEvent>().unwrap().emit();
            }
            Ok(())
        })
        .inner?;
        if let Some(trs) = &self.language_select_open {
            let mut close = false;
            let ctx = env.get::<egui::Context>().unwrap();
            egui::Window::new(app_st.text_core("ui_main_language-select")?)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        for tr in trs {
                            if ui
                                .add_sized(
                                    vec2(64., 32.),
                                    egui::Button::new(tr.0.to_string().to_uppercase()),
                                )
                                .clicked()
                            {
                                env.get::<AppSwitchTranslationEvent>()
                                    .unwrap()
                                    .emit(tr.clone());
                                close = true;
                                break;
                            }
                        }
                        Ok(())
                    })
                    .inner
                })
                .map(|x| x.inner.transpose())
                .transpose()?;
            if close {
                self.language_select_open = None;
            }
        }
        Ok(())
    }
}
