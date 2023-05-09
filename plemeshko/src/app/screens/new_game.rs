use anyhow::Ok;
use egui::TextEdit;

use crate::{app::widgets::Widget, state::has::HasTexts};

use super::{AppNewGameEvent, AppScreen, AppScreenTransitionEvent};

pub struct NewGameScreen {
    name: String,
}

impl NewGameScreen {
    pub fn new() -> Self {
        NewGameScreen {
            name: String::new(),
        }
    }
}

impl Widget for NewGameScreen {
    type Response = ();

    fn ui(
        &mut self,
        env: &mut crate::app::env::Env<'_>,
        ui: &mut egui::Ui,
    ) -> anyhow::Result<Self::Response> {
        let app_st = env.app_state();
        ui.horizontal(|ui| {
            ui.label(app_st.text_core("ui_new-game_name")?);
            ui.add(
                TextEdit::singleline(&mut self.name)
                    .hint_text(app_st.text_core("ui_new-game_name-hint")?),
            );
            Ok(())
        })
        .inner?;
        let valid_name = self
            .name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_');
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    !self.name.is_empty() && valid_name,
                    egui::Button::new(app_st.text_core("ui_new-game_start")?),
                )
                .clicked()
            {
                env.get::<AppNewGameEvent>()
                    .unwrap()
                    .emit(self.name.clone());
                env.get::<AppScreenTransitionEvent>()
                    .unwrap()
                    .emit(AppScreen::Main);
            }
            if ui.button(app_st.text_core("ui_generic_return")?).clicked() {
                env.get::<AppScreenTransitionEvent>()
                    .unwrap()
                    .emit(AppScreen::Menu);
            }
            Ok(())
        })
        .inner?;
        if self.name.is_empty() {
            ui.label(app_st.text_core("ui_new-game_err-empty")?);
        }
        if !valid_name {
            ui.label(app_st.text_core("ui_new-game_err-invalid")?);
        }
        Ok(())
    }
}
