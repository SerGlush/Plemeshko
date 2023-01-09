use crate::server::{
    config::resource::{Resource, ResourceId},
    Sim,
};
use egui::*;
use plegine::config::ConfigId;

pub struct App {
    current_panel: i64,
}

impl App {
    pub fn new() -> Self {
        App { current_panel: 0 }
    }

    pub fn update(&mut self, sim: &mut Sim) {}

    pub fn gui(&mut self, context: &egui::Context, sim: &mut Sim) {
        CentralPanel::default().show(context, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Main").clicked() {
                    self.current_panel = 0;
                }
                if ui.button("Erections").clicked() {
                    self.current_panel = 1;
                }
                if ui.button("Placeholder").clicked() {
                    self.current_panel = 2;
                }
            });
            match self.current_panel {
                0 => {
                    ui.label(format!(
                        "Population: {}",
                        sim.depot
                            .get(&ResourceId::new("human"))
                            .map(Clone::clone)
                            .unwrap_or_default()
                    ));
                }
                _ => (),
            }
        });
    }
}
