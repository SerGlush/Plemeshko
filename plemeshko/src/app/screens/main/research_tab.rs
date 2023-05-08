use std::fmt::Write;

use anyhow::{Ok, Result};
use egui::{Color32, ProgressBar, WidgetText};

use crate::{
    app::{
        env::Env,
        widgets::{Tab, Widget},
    },
    sim::config::technology::Technology,
    state::{
        has::{HasSimMutex, HasTexts},
        AppState,
    },
};

pub struct MainScreenResearchTab;

impl MainScreenResearchTab {
    pub fn new() -> Self {
        MainScreenResearchTab
    }
}

impl Tab for MainScreenResearchTab {
    fn header(&self, env: &Env<'_>) -> Result<WidgetText> {
        let app_st = env.get::<AppState>().unwrap();
        Ok(app_st.text_core("ui_main_research_header")?.into())
    }
}

impl Widget for MainScreenResearchTab {
    type Response = ();

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut egui::Ui) -> Result<()> {
        let app_st = env.app_state();
        let shared_comps = env.shared_components();
        let mut sim_guard = app_st.lock_sim();
        let sim = sim_guard.as_mut().unwrap();
        let ctx = env.get::<egui::Context>().unwrap();

        let mut current_research_text = app_st.text_core("ui_main_research_current")?.into_owned();
        if let Some((id, progress)) = sim.research.current() {
            write!(
                current_research_text,
                ": {}",
                app_st.text(&shared_comps.config(id)?.info.name)?
            )?;
            ui.horizontal(|ui| {
                ui.label(current_research_text);
                ui.add(ProgressBar::new(
                    progress as f32 / shared_comps.config(id)?.cost as f32,
                ));
                Ok(())
            })
            .inner?;
        } else {
            write!(
                current_research_text,
                ": {}",
                app_st.text_core("ui_generic_nothing")?
            )?;
            ui.label(current_research_text);
        }
        ui.separator();

        for technology in shared_comps.iter_configs::<Technology>() {
            let (id, technology) = technology?;
            let mut button = egui::ImageButton::new(
                app_st
                    .texture(technology.info.icon.texture)?
                    .texture_id(ctx),
                egui::vec2(64.0, 64.0),
            );
            if let Some(uv) = technology.info.icon.uv {
                button = button.uv(uv);
            }
            let is_researched = sim.research.is_researched(id);
            let prerequisites_satisfied = technology.prerequisites_satisfied();
            button = button.tint(match (is_researched, prerequisites_satisfied) {
                (false, false) => Color32::from_rgb(255, 120, 120),
                (false, true) => Color32::from_rgb(155, 155, 155),
                (true, _) => Color32::WHITE,
            });
            if let Some((current_id, _)) = sim.research.current() {
                button = button.frame(current_id == id)
            } else {
                button = button.frame(false);
            }
            if ui.add(button).clicked() && !is_researched && prerequisites_satisfied {
                sim.research.start(id);
            }
        }

        Ok(())
    }
}
