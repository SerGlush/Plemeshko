use crate::server::Sim;
use egui::*;

use super::error::GuiResult;

const HUMAN_ID: &'static str = "human";

pub struct App {
    current_panel: i64,
}

impl App {
    pub fn new() -> Self {
        App { current_panel: 0 }
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
                if ui.button("Placeholder").clicked() {
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
                _ => (),
            }
        });

        Ok(())
    }
}
