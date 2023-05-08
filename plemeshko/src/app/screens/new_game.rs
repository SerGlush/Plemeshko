use egui::TextEdit;

use crate::app::widgets::Widget;

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
        ui.horizontal(|ui| {
            ui.label("Game name:");
            ui.add(
                TextEdit::singleline(&mut self.name).hint_text("Letters, digits and underscores"),
            );
        });
        let valid_name = self
            .name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_');
        ui.horizontal(|ui| {
            if ui
                .add_enabled(valid_name, egui::Button::new("Start"))
                .clicked()
            {
                env.get::<AppNewGameEvent>()
                    .unwrap()
                    .emit(self.name.clone());
                env.get::<AppScreenTransitionEvent>()
                    .unwrap()
                    .emit(AppScreen::Main);
            }
            if ui.button("Return").clicked() {
                env.get::<AppScreenTransitionEvent>()
                    .unwrap()
                    .emit(AppScreen::Menu);
            }
        });
        if !valid_name {
            ui.label("Name contains invalid characters!");
        }
        Ok(())
    }
}
