use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Write,
};

use anyhow::Result;
use fluent::FluentArgs;

use crate::{
    app::{
        env::Env,
        screens::AppSaveEvent,
        widgets::{Tab, Widget},
    },
    sim::{config::resource::ResourceId, units::ResourceAmount},
    state::{
        has::{HasSimMutex, HasTexts},
        AppState,
    },
};

pub struct MainScreenInfoTab {
    previous_depot: HashMap<ResourceId, ResourceAmount>,
    depot_change: HashMap<ResourceId, ResourceAmount>,
}

impl MainScreenInfoTab {
    pub fn new() -> Self {
        MainScreenInfoTab {
            previous_depot: HashMap::new(),
            depot_change: HashMap::new(),
        }
    }
}

impl Widget for MainScreenInfoTab {
    type Response = ();

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut egui::Ui) -> Result<Self::Response> {
        let app_st = env.app_state();
        let shared_comps = env.shared_components();
        if ui.button("Save").clicked() {
            env.get::<AppSaveEvent>().unwrap().emit();
        }
        let mut sim_guard = app_st.lock_sim();
        let sim = sim_guard.as_mut().unwrap();
        {
            let mut args = FluentArgs::new();
            let mut population_count_text = sim
                .depot
                .get(&app_st.shared.human_id)
                .map(Clone::clone)
                .unwrap_or_default()
                .to_string();
            if let Some(change) = self.depot_change.get(&app_st.shared.human_id) {
                population_count_text += " (";
                if change.0 > 0 {
                    population_count_text.push('+');
                }
                write!(population_count_text, "{})", change.0)?;
            }
            args.set("population", population_count_text);
            ui.label(app_st.text_core_fmt("ui_main_info_population", &args)?);
            ui.label(sim.nutrition.to_string());
        }
        for (&id, &value) in sim.depot.iter() {
            if id != app_st.shared.human_id {
                let res = shared_comps.config(id)?;
                let mut res_info_text = format!("{} : {value}", app_st.text(&res.name)?);
                if let Some(change) = self.depot_change.get(&id) {
                    res_info_text += " (";
                    if change.0 > 0 {
                        res_info_text.push('+');
                    }
                    write!(res_info_text, "{})", change.0)?;
                }
                ui.label(res_info_text);
            }
        }
        if sim.handle_state_changed() {
            self.depot_change.clear();
            for (&id, &current_value) in &sim.depot {
                let change =
                    current_value - self.previous_depot.get(&id).copied().unwrap_or_default();
                if change.0 != 0 {
                    self.depot_change.insert(id, change);
                }
            }
            for (&id, &prev_value) in &self.previous_depot {
                match self.depot_change.entry(id) {
                    Entry::Vacant(vacant) => {
                        let change = sim.depot.get(&id).copied().unwrap_or_default() - prev_value;
                        if change.0 != 0 {
                            vacant.insert(change);
                        }
                    }
                    Entry::Occupied(_) => {}
                }
            }
            self.previous_depot.clone_from(&sim.depot);
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
