use anyhow::Result;
use egui::WidgetText;

use crate::{
    app::{
        env::Env,
        widgets::{Tab, Widget},
    },
    state::{
        has::{HasSimMutex, HasTexts},
        AppState,
    },
    util::cor::Cor,
};

pub struct MainScreenDebugTab {
    spawn_resource_name: String,
    spawn_resource_value: String,
}

impl MainScreenDebugTab {
    pub fn new() -> Self {
        MainScreenDebugTab {
            spawn_resource_name: "human".to_string(),
            spawn_resource_value: "10".to_string(),
        }
    }
}

impl Tab for MainScreenDebugTab {
    fn header(&self, env: &Env<'_>) -> Result<WidgetText> {
        let app_st = env.get::<AppState>().unwrap();
        Ok(app_st.text_core("ui_main_debug_header")?.into())
    }
}

impl Widget for MainScreenDebugTab {
    type Response = ();

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut egui::Ui) -> Result<()> {
        let app_st = env.app_state();
        let shared_comps = env.shared_components();
        let mut sim_guard = app_st.lock_sim();
        let sim = sim_guard.as_mut().unwrap();
        ui.text_edit_singleline(&mut self.spawn_resource_name);
        ui.text_edit_singleline(&mut self.spawn_resource_value);
        if ui
            .button(app_st.text_core("ui_main_debug_spawn-resources")?)
            .clicked()
        {
            sim.depot.cor_put(
                &shared_comps
                    .core()?
                    .configs
                    .id_from_raw(self.spawn_resource_name.as_str())
                    .unwrap()
                    .in_core(),
                self.spawn_resource_value.parse().unwrap(),
            );
        }
        Ok(())
    }
}
