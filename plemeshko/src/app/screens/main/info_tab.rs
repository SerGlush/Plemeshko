use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Write,
};

use anyhow::{Ok, Result};
use egui::{vec2, Color32};
use fluent::FluentArgs;

use crate::{
    app::{
        env::Env,
        screens::AppSaveEvent,
        util::draw_icon_with_tooltip,
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
    previous_nutrition: Option<i64>,
    nutrition_change: i64,
}

impl MainScreenInfoTab {
    pub fn new() -> Self {
        MainScreenInfoTab {
            previous_depot: HashMap::new(),
            depot_change: HashMap::new(),
            previous_nutrition: None,
            nutrition_change: 0,
        }
    }
}

impl Widget for MainScreenInfoTab {
    type Response = ();

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut egui::Ui) -> Result<Self::Response> {
        let app_st = env.app_state();
        let shared_comps = env.shared_components();
        ui.horizontal(|ui| {
            ui.label(app_st.session.as_ref().unwrap());
            if ui.button(app_st.text_core("ui_main_info_save")?).clicked() {
                env.get::<AppSaveEvent>().unwrap().emit();
            }
            Ok(())
        })
        .inner?;
        let mut sim_guard = app_st.lock_sim();
        let sim = sim_guard.as_mut().unwrap();
        ui.label(app_st.text_core("ui_main_info_stats")?);
        ui.indent("stats", |ui| {
            {
                let mut args = FluentArgs::new();
                let mut population_count_text = sim
                    .depot
                    .get(&app_st.shared.human_id)
                    .copied()
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
            }
            {
                let mut args = FluentArgs::new();
                let mut nutrition = sim.nutrition.to_string();
                if self.nutrition_change != 0 {
                    nutrition += " (";
                    if self.nutrition_change > 0 {
                        nutrition.push('+');
                    }
                    write!(nutrition, "{})", self.nutrition_change)?;
                }
                args.set("nutrition", nutrition);
                ui.label(app_st.text_core_fmt("ui_main_info_nutrition", &args)?);
            }
            Ok(())
        })
        .inner?;
        let ctx = env.get::<egui::Context>().unwrap();
        ui.label(app_st.text_core("ui_main_info_resources")?);
        ui.indent("resources", |ui| {
            for (&id, &value) in sim.depot.iter() {
                if id != app_st.shared.human_id {
                    let res = shared_comps.config(id)?;
                    let change = self.depot_change.get(&id);
                    ui.horizontal(|ui| {
                        draw_icon_with_tooltip(
                            app_st,
                            ctx,
                            ui,
                            &res.info,
                            vec2(32.0, 32.0),
                            |i| {
                                i.tint(match change {
                                    Some(c) if c.0 > 0 => Color32::from_rgb(200, 200, 255),
                                    Some(c) if c.0 < 0 => Color32::from_rgb(255, 200, 200),
                                    _ => Color32::WHITE,
                                })
                            },
                            |_| (),
                        )?;
                        let mut res_info_text = value.to_string();
                        if let Some(change) = change {
                            res_info_text += " (";
                            if change.0 > 0 {
                                res_info_text.push('+');
                            }
                            write!(res_info_text, "{})", change.0)?;
                        }
                        ui.label(res_info_text);
                        Ok(())
                    })
                    .inner?;
                }
            }
            Ok(())
        })
        .inner?;
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

            if let Some(previous_nutrition) = &mut self.previous_nutrition {
                self.nutrition_change = sim.nutrition - *previous_nutrition;
            }
            self.previous_nutrition = Some(sim.nutrition);
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
