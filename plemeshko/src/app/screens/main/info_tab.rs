use anyhow::Result;
use fluent::FluentArgs;

use crate::{
    app::{
        env::Env,
        widgets::{Tab, Widget},
    },
    state::{
        has::{HasSimMutex, HasTexts},
        AppState,
    },
};

pub struct MainScreenInfoTab;

impl MainScreenInfoTab {
    pub fn new() -> Self {
        MainScreenInfoTab
    }
}

impl Widget for MainScreenInfoTab {
    type Response = ();

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut egui::Ui) -> Result<Self::Response> {
        let app_st = env.app_state();
        let shared_comps = env.shared_components();
        let mut sim_guard = app_st.lock_sim();
        let sim = sim_guard.as_mut().unwrap();
        {
            let mut args = FluentArgs::new();
            args.set(
                "population",
                sim.depot
                    .get(&app_st.shared.human_id)
                    .map(Clone::clone)
                    .unwrap_or_default()
                    .to_string(),
            );
            ui.label(app_st.text_core_fmt("ui_main_info_population", &args)?);
            ui.label(sim.nutrition.to_string());
        }
        for (&id, value) in sim.depot.iter() {
            if id != app_st.shared.human_id {
                let res = shared_comps.config(id)?;
                ui.label(format!("{} : {value}", app_st.text(&res.name)?));
            }
        }
        Ok(())
    }
}

impl Tab for MainScreenInfoTab {
    fn header(&self, env: &Env<'_>) -> Result<egui::WidgetText> {
        let app_st = env.get::<AppState>().unwrap();
        Ok(app_st.text_core("ui_main_info_header")?.into())
    }
}
