use anyhow::{Ok, Result};
use egui::{vec2, Button};
use fluent::FluentArgs;

use crate::{
    app::{env::Env, widgets::Widget},
    state::{has::HasTexts, save::SaveMetadata},
};

use super::{AppLoadEvent, AppScreen, AppScreenTransitionEvent};

struct LoadScreenId;

pub struct LoadScreen {
    saves: Option<Vec<(String, SaveMetadata)>>,
}

impl LoadScreen {
    pub fn new() -> Self {
        LoadScreen { saves: None }
    }
}

impl Widget for LoadScreen {
    type Response = ();

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut egui::Ui) -> Result<Self::Response> {
        if self.saves.is_none() {
            let saves = crate::state::save::saves()?;
            if saves.len() == 1 {
                env.get::<AppLoadEvent>().unwrap().emit(saves[0].0.clone());
                env.get::<AppScreenTransitionEvent>()
                    .unwrap()
                    .emit(AppScreen::Main);
                return Ok(());
            }
            self.saves = Some(saves);
        }
        let app_st = env.app_state();
        let mut ev_refresh = false;
        let saves = self.saves.as_ref().unwrap();
        let btnsz = vec2(
            (ui.available_width() / 4.).min(300.),
            (ui.available_height() / 6.).min(200.),
        );
        for (name, meta) in saves {
            let mut args = FluentArgs::new();
            args.set("save_name", name.to_owned());
            args.set("saved_date", meta.saved_date.to_string());
            args.set("play_time", meta.play_time.as_secs());
            if ui
                .add_sized(
                    btnsz,
                    Button::new(app_st.text_core_fmt("ui_load_load", &args)?),
                )
                .clicked()
            {
                env.get::<AppLoadEvent>().unwrap().emit(name.clone());
                env.get::<AppScreenTransitionEvent>()
                    .unwrap()
                    .emit(AppScreen::Main);
                ev_refresh = true;
            }
        }
        ui.add_space(btnsz.y * 0.5);
        if ui
            .add_sized(
                vec2(btnsz.x, btnsz.y * 0.5),
                Button::new(app_st.text_core("ui_generic_return")?),
            )
            .clicked()
        {
            env.get::<AppScreenTransitionEvent>()
                .unwrap()
                .emit(AppScreen::Menu);
            ev_refresh = true;
        }
        if ev_refresh {
            self.saves = None;
        }
        Ok(())
    }
}
