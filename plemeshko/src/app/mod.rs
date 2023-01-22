use std::collections::HashMap;

use crate::{
    env::AppEnv,
    sim::{
        config::{method::MethodId, transport::TransportId, transport_group::{TransportGroupId, self}},
        erection::Erection,
        Sim,
    },
    util::cor::Cor,
};
use anyhow::Ok;
use egui::*;
use fluent::FluentArgs;

pub struct App {
    current_panel: i64,
    spawn_resource_name: String,
    spawn_resource_value: String,
    erection_builder_name: String,
    erection_builder_transport: HashMap<TransportGroupId, TransportId>,
    erection_builder_methods: Vec<MethodId>,
}

pub fn draw_erection(erection: &Erection, ui: &mut Ui, env: &AppEnv) -> anyhow::Result<()> {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", erection.name()));
        for &transport_id in erection.transport().values() {
            let transport = config_get!(env.configs(), transport_id);
            let transport_group = config_get!(env.configs(), transport.group);
            ui.label(env.text(&transport.name)?)
                .on_hover_text(format!("Transport Group: {}\nTransport Capacity: {}", env.text(&transport_group.name)?, transport.capacity));
        }
        Ok(())
    })
    .inner?;
    for selected_method in erection.methods().iter() {
        let method = config_get!(env.configs(), selected_method.id);
        ui.horizontal(|ui| {
            ui.label(env.text(&method.name)?);
            for setting in selected_method.settings.iter() {
                let setting_group = config_get!(env.configs(), setting.group);
                ui.label(env.text(&setting_group.settings[setting.index].name)?);
            }
            Ok(())
        })
        .inner?;
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

    pub fn gui(
        &mut self,
        context: &egui::Context,
        sim: &mut Sim,
        env: &mut AppEnv,
    ) -> anyhow::Result<()> {
        CentralPanel::default()
            .show(context, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(env.text("ui_main")?).clicked() {
                        self.current_panel = 0;
                    }
                    if ui.button(env.text("ui_erections")?).clicked() {
                        self.current_panel = 1;
                    }
                    if ui.button(env.text("ui_debug")?).clicked() {
                        self.current_panel = 2;
                    }
                    Ok(())
                })
                .inner?;
                match self.current_panel {
                    0 => {
                        let mut args = FluentArgs::new();
                        args.set(
                            "population",
                            sim.depot
                                .get(&env.shared.human_id)
                                .map(Clone::clone)
                                .unwrap_or_default()
                                .to_string(),
                        );
                        ui.label(env.text_fmt("ui_main_population", &args)?);
                        for (&id, value) in sim.depot.iter() {
                            if id != env.shared.human_id {
                                let res = config_get!(env.configs(), id);
                                ui.label(format!("{} : {value}", env.text(&res.name)?));
                            }
                        }
                    }
                    1 => {
                        if ui.button(env.text("ui_erections_create")?).clicked() {
                            Window::new(env.text("ui_erections_builder")?).show(context, |ui| {
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
                        if ui.button(env.text("ui_debug_spawn-resources")?).clicked() {
                            sim.depot.cor_put(
                                &env.configs()
                                    .indexer
                                    .get_id(self.spawn_resource_name.clone())
                                    .unwrap(),
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
