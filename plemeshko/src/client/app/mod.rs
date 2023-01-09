use crate::server::{
    config::resource::{storage::Cor, Resource, ResourceId},
    erection::Erection,
    Sim,
};
use egui::*;

use super::error::GuiResult;

const HUMAN_ID: &'static str = "human";

pub struct App {
    current_panel: i64,
    spawn_resource_name: String,
    spawn_resource_value: String,
}

pub fn draw_erection(ui: &mut Ui, erection: &Erection) {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", erection.name()))
    });
}

impl App {
    pub fn new() -> Self {
        App {
            current_panel: 0,
            spawn_resource_name: "human".to_string(),
            spawn_resource_value: "10".to_string(),
        }
    }

    pub fn update(&mut self, _sim: &mut Sim) -> GuiResult<()> {
        Ok(())
    }

    pub fn gui(&mut self, context: &egui::Context, sim: &mut Sim) -> GuiResult<()> {
        CentralPanel::default().show(context, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Main").clicked() {
                    self.current_panel = 0;
                }
                if ui.button("Erections").clicked() {
                    self.current_panel = 1;
                }
                if ui.button("Debug").clicked() {
                    self.current_panel = 2;
                }
            });
            match self.current_panel {
                0 => {
                    ui.label(format!(
                        "Population: {}",
                        sim.depot
                            .get(HUMAN_ID)
                            .map(Clone::clone)
                            .unwrap_or_default()
                    ));
                    for (id, value) in sim.depot.iter() {
                        if id.as_str() == HUMAN_ID {
                            continue;
                        }
                        ui.label(format!("{} : {}", id, value));
                    }
                }
                1 => {
                    for (erection) in sim.erections.iter() {
                        draw_erection(ui, erection);
                    }
                }
                2 => {
                    ui.text_edit_singleline(&mut self.spawn_resource_name);
                    ui.text_edit_singleline(&mut self.spawn_resource_value);
                    if ui.button("Spawn resource").clicked() {
                        sim.depot.cor_put(
                            &ResourceId::new(self.spawn_resource_name.clone()),
                            self.spawn_resource_value.parse().unwrap(),
                        );
                    }
                }
                _ => (),
            }
        });

        Ok(())
    }
}
