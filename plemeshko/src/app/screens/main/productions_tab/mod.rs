use anyhow::{Ok, Result};
use egui::{vec2, Button};

use crate::{
    app::{
        env::Env,
        util::ConfigIteratorExt,
        widgets::{PersistentWindow, Tab, Widget},
    },
    sim::production::Production,
    state::{
        components::SharedComponents,
        has::{HasSimMutex, HasTexts},
        AppState,
    },
};

use self::production_menu::ProductionBuilder;

mod production_menu;

pub struct MainScreenProductionsTab {
    production_menu: PersistentWindow<ProductionBuilder, ProductionBuilder>,
}

impl MainScreenProductionsTab {
    pub fn new() -> Self {
        MainScreenProductionsTab {
            production_menu: PersistentWindow::new(ProductionBuilder::new()),
        }
    }
}

fn ui_production(
    app_st: &AppState,
    shared_comps: &SharedComponents,
    ui: &mut egui::Ui,
    production: &mut Production,
) -> Result<()> {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", production.name()));
        if ui
            .add(Button::new("+").min_size(vec2(16.0, 16.0)))
            .clicked()
        {
            if production.count() == production.active() {
                production.set_count(production.count() + 1);
                production.set_active(production.active() + 1);
            } else {
                production.set_active(production.active() + 1);
            }
        }
        ui.label(format!("{}/{}", production.active(), production.count()));
        if ui
            .add(Button::new("-").min_size(vec2(16.0, 16.0)))
            .clicked()
        {
            production.set_active(production.active() - 1);
        }
        for transport in production.transport().values().configs(shared_comps) {
            let transport_group = shared_comps.config(transport.group)?;
            ui.label(app_st.text(&transport.name)?)
                .on_hover_text(format!(
                    "Transport Group: {}\nTransport Capacity: {}",
                    app_st.text(&transport_group.name)?,
                    transport.capacity
                ));
        }
        Ok(())
    })
    .inner?;
    for selected_method in production.methods() {
        let method = shared_comps.config(selected_method.id)?;
        ui.horizontal(|ui| {
            ui.label(app_st.text(&method.name)?);
            for setting in selected_method.settings.configs(shared_comps) {
                ui.label(app_st.text(&setting.name)?);
            }
            Ok(())
        })
        .inner?;
    }
    Ok(())
}

impl Widget for MainScreenProductionsTab {
    type Response = ();

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut egui::Ui) -> Result<Self::Response> {
        if ui
            .button(
                env.app_state()
                    .text_core("ui_main_productions_open-builder")?,
            )
            .clicked()
        {
            self.production_menu.is_open = true;
        }
        self.production_menu.ui(env, ui)?;
        let app_st = env.app_state();
        let shared_comps = env.shared_components();
        let mut sim_guard = app_st.lock_sim();
        let sim = sim_guard.as_mut().unwrap();
        for production in &mut sim.productions {
            ui_production(app_st, shared_comps, ui, production)?;
        }
        Ok(())
    }
}

impl Tab for MainScreenProductionsTab {
    fn header(&self, env: &Env<'_>) -> Result<egui::WidgetText> {
        env.app_state()
            .text_core("ui_main_productions_header")
            .map(Into::into)
    }
}
