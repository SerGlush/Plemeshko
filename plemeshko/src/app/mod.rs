use std::collections::HashMap;

use crate::{
    env::Env,
    sim::{
        config::{
            method::MethodId,
            resource::{storage::Cor, ResourceId},
            transport::TransportId,
            transport_group::TransportGroupId,
        },
        erection::Erection,
        Sim, RESOURCE_ID_HUMAN,
    },
};
use anyhow::Ok;
use egui::*;

pub struct App {
    current_panel: i64,
    spawn_resource_name: String,
    spawn_resource_value: String,
    erection_builder_name: String,
    erection_builder_transport: HashMap<TransportGroupId, TransportId>,
    erection_builder_methods: Vec<MethodId>,
}

pub fn draw_erection(erection: &Erection, ui: &mut Ui, env: &Env) -> anyhow::Result<()> {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", erection.name()));
        for transport_id in erection.transport().values() {
            let transport = config_get!(env.configs, transport_id);
            ui.label(transport_id.as_str())
                .on_hover_text(format!("transport capacity: {}", transport.capacity));
        }
        Ok(())
    })
    .inner?;
    for method in erection.methods().iter() {
        ui.horizontal(|ui| ui.label(method.id.as_str()).on_hover_text("Placeholder"));
    }
    Ok(())
}

impl App {
    pub fn new() -> Self {
        App {
            current_panel: 0,
            spawn_resource_name: "human".to_string(),
            spawn_resource_value: "10".to_string(),
            erection_builder_name: "input name".to_string(),
            erection_builder_transport: HashMap::<TransportGroupId, TransportId>::new(),
            erection_builder_methods: Vec::<MethodId>::new(),
        }
    }

    pub fn update(&mut self, _sim: &mut Sim) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn gui(&mut self, context: &egui::Context, sim: &mut Sim, env: &Env) -> anyhow::Result<()> {
        CentralPanel::default()
            .show(context, |ui| {
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
                                .get(RESOURCE_ID_HUMAN)
                                .map(Clone::clone)
                                .unwrap_or_default()
                        ));
                        for (id, value) in sim.depot.iter() {
                            if id.as_str() != RESOURCE_ID_HUMAN {
                                ui.label(format!("{id} : {value}"));
                            }
                        }
                    }
                    1 => {
                        if ui.button("Create Erection").clicked() {
                            Window::new("Erection Builder").show(context, |ui| {
                                ui.horizontal(|ui| {
                                    ui.text_edit_singleline(&mut self.erection_builder_name);
                                })
                                //for method in self.erection_builder_methods {
                                //    ui.
                                //}
                            });
                        }
                        for erection in sim.erections.iter() {
                            draw_erection(erection, ui, env)?;
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
                Ok(())
            })
            .inner?;

        Ok(())
    }
}
