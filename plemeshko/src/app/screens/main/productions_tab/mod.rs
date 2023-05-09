use anyhow::{Ok, Result};
use egui::{vec2, Button};

use crate::{
    app::{
        env::Env,
        util::{on_using_modifiers, ConfigIteratorExt},
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
    production_index: usize,
    productions: &mut Vec<Production>,
) -> Result<()> {
    ui.separator();
    let removed = ui
        .horizontal(|ui| {
            let production = &mut productions[production_index];

            ui.strong(format!("{}:", production.name()));
            on_using_modifiers(
                &ui.add(Button::new("+").min_size(vec2(16.0, 16.0))),
                egui::Response::clicked,
                |m| {
                    let delta = m.elim(1, 10, 100);
                    let new_active = production.active() + delta;
                    if new_active > production.count() {
                        production.set_count(new_active);
                    }
                    production.set_active(new_active);
                },
            );
            ui.label(format!("{}/{}", production.active(), production.count()));

            on_using_modifiers(
                &ui.add(Button::new("-").min_size(vec2(16.0, 16.0))),
                egui::Response::clicked,
                |m| {
                    let delta = m.elim(1, 10, 100);
                    let new_active = if production.active() > delta {
                        production.active() - delta
                    } else {
                        0
                    };
                    production.set_active(new_active);
                },
            );

            if ui.button(app_st.text_core("ui_generic_delete")?).clicked() {
                productions.remove(production_index);
                return Ok(true);
            }

            let production = &mut productions[production_index];

            for transport in production.transport().values().configs(shared_comps) {
                let transport_group = shared_comps.config(transport.group)?;
                ui.label(app_st.text(&transport.info.name)?)
                    .on_hover_text(format!(
                        "Transport Group: {}\nTransport Capacity: {}",
                        app_st.text(&transport_group.name)?,
                        transport.capacity
                    ));
            }
            Ok(false)
        })
        .inner?;

    if removed {
        return Ok(());
    }

    for selected_method in productions[production_index].methods() {
        let method = shared_comps.config(selected_method.id)?;
        ui.horizontal(|ui| {
            ui.label(app_st.text(&method.info.name)?);
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
        for production_index in 0..sim.productions.len() {
            ui_production(
                app_st,
                shared_comps,
                ui,
                production_index,
                &mut sim.productions,
            )?;
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
