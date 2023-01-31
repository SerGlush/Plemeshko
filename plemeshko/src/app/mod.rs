mod util;

use std::collections::HashMap;

use anyhow::{Ok, Result};
use egui::*;
use fluent::FluentArgs;

use crate::{
    sim::{
        config::{
            method_group::MethodGroup, production_method::SelectedMethod, resource::Resource,
            transport_group::TransportGroupId, transport_method::TransportId,
        },
        production::Production,
    },
    state::{components::SharedComponents, AppState},
    util::cor::Cor,
};

use util::*;

pub struct ProductionBuilder {
    window_is_open: bool,

    production_name: String,
    production_transport: HashMap<TransportGroupId, (TransportId, bool)>,
    production_methods: Vec<SelectedMethod>,
}

impl ProductionBuilder {
    pub fn new() -> ProductionBuilder {
        ProductionBuilder {
            production_name: "input name".to_string(),
            production_transport: HashMap::<TransportGroupId, (TransportId, bool)>::new(),
            production_methods: Vec::<SelectedMethod>::new(),
            window_is_open: false,
        }
    }

    pub fn open(&mut self) {
        self.window_is_open = true;
    }

    pub fn close(&mut self) {
        self.window_is_open = false;
    }

    pub fn draw(
        &mut self,
        st: &AppState,
        shared_comps: &SharedComponents,
        egui_ctx: &egui::Context,
    ) -> Result<()> {
        let mut window_is_open = self.window_is_open;
        if window_is_open {
            Window::new(st.text_core("ui_production-builder_title")?)
                .open(&mut window_is_open)
                .show(egui_ctx, |ui| {
                    self.add_window_contents(st, shared_comps, ui)
                })
                .unwrap()
                .inner
                .transpose()?;
        }

        self.window_is_open = window_is_open;
        Ok(())
    }

    fn add_window_contents(
        &mut self,
        st: &AppState,
        shared_comps: &SharedComponents,
        ui: &mut Ui,
    ) -> Result<()> {
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.production_name);

            self.production_transport
                .values_mut()
                .try_for_each(|value| {
                    let selected_transport = shared_comps.config(value.0)?;
                    value.1 = false;

                    let selected_transport_name = st.text(&selected_transport.name)?;
                    ComboBox::from_id_source(selected_transport_name.as_ref())
                        .selected_text(selected_transport_name.as_ref())
                        .show_ui(ui, |ui| {
                            let transport_group = shared_comps.config(selected_transport.group)?;
                            for (transport_id, transport) in
                                transport_group.transports.configs_with_ids(shared_comps)
                            {
                                let transport_name = st.text(&transport.name)?;

                                if ui.selectable_label(false, transport_name).clicked() {
                                    value.0 = transport_id;
                                }
                            }

                            Ok(())
                        });

                    Ok(())
                })?;

            for selected_method in &self.production_methods {
                for setting in selected_method.settings.configs(shared_comps) {
                    let mut check_resource_group = |resource: &Resource| {
                        let mut new_group_check: bool = true;
                        for (key, value) in self.production_transport.iter_mut() {
                            if *key == resource.transport_group {
                                value.1 = true;
                                new_group_check = false;
                            }
                        }

                        if new_group_check {
                            let transport_id = shared_comps
                                .config(resource.transport_group)?
                                .transports
                                .configs_with_ids(shared_comps)
                                .find(|(_, tr)| {
                                    tr.ui_priority == 0 && tr.group == resource.transport_group
                                })
                                .unwrap()
                                .0;
                            self.production_transport
                                .insert(resource.transport_group, (transport_id, true));
                        }

                        Ok(())
                    };

                    setting
                        .resource_io
                        .input
                        .keys()
                        .configs(shared_comps)
                        .try_for_each(&mut check_resource_group)?;
                    setting
                        .resource_io
                        .output
                        .keys()
                        .configs(shared_comps)
                        .try_for_each(&mut check_resource_group)?;
                }
            }

            Ok(())
        });

        draw_iter_indexed(ui, self.production_methods.iter_mut(), |ui, method| {
            ui.horizontal(|ui| {
                let method_name = st.text(&shared_comps.config(method.id)?.name)?;
                ui.label(method_name.as_ref());
                for selected_setting_id in &mut method.settings {
                    let selected_setting = shared_comps.config(*selected_setting_id)?;
                    let setting_group = shared_comps.config(selected_setting.group)?;
                    let selected_setting_name = st.text(&selected_setting.name)?;
                    ComboBox::from_id_source(&selected_setting_name)
                        .width(200.0)
                        .selected_text(selected_setting_name)
                        .show_ui(ui, |ui| {
                            for (setting_id, setting) in
                                setting_group.settings.configs_with_ids(shared_comps)
                            {
                                if ui.button(st.text(&setting.name)?).clicked() {
                                    *selected_setting_id = setting_id;
                                }
                            }
                            Ok(())
                        })
                        .inner
                        .transpose()?;
                }
                Ok(())
            })
            .inner
        })?;

        ui.menu_button(st.text_core("ui_production-builder_add-method")?, |ui| {
            for method_group in shared_comps.iter_configs::<MethodGroup>() {
                let method_group = method_group?.1;
                ui.menu_button(st.text(&method_group.name)?, |ui| {
                    for (method_id, method) in method_group.variants.configs_with_ids(shared_comps)
                    {
                        if ui.button(st.text(&method.name)?).clicked() {
                            let selected_method =
                                SelectedMethod::new(shared_comps, method_id, None)?;
                            self.production_methods.push(selected_method.clone());
                        }
                    }
                    Ok(())
                });
            }
            Ok(())
        });

        if ui
            .button(st.text_core("ui_production-builder_create-production")?)
            .clicked()
        {
            let mut sim_guard = st.shared.sim.lock().unwrap();
            let sim = sim_guard.as_mut().unwrap();

            sim.productions.push(Production::new(
                shared_comps,
                self.production_name.clone(),
                self.production_methods.clone(),
                self.production_transport
                    .iter()
                    .map(|(&key, &(value, _))| (key, value))
                    .collect(),
            )?);
        }
        Ok(())
    }
}

pub struct App {
    current_panel: i64,
    spawn_resource_name: String,
    spawn_resource_value: String,
    production_builder: ProductionBuilder,
}

pub fn draw_production(
    st: &AppState,
    shared_comps: &SharedComponents,
    ui: &mut Ui,
    production: &Production,
) -> anyhow::Result<()> {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", production.name()));
        for transport in production.transport().values().configs(shared_comps) {
            let transport_group = shared_comps.config(transport.group)?;
            ui.label(st.text(&transport.name)?).on_hover_text(format!(
                "Transport Group: {}\nTransport Capacity: {}",
                st.text(&transport_group.name)?,
                transport.capacity
            ));
        }
        Ok(())
    })
    .inner?;
    for selected_method in production.methods() {
        let method = shared_comps.config(selected_method.id)?;
        ui.horizontal(|ui| {
            ui.label(st.text(&method.name)?);
            for setting in selected_method.settings.configs(shared_comps) {
                ui.label(st.text(&setting.name)?);
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
            production_builder: ProductionBuilder::new(),
        }
    }

    pub fn update(&mut self, _st: &mut AppState) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn gui(&mut self, st: &mut AppState, egui_ctx: &egui::Context) -> anyhow::Result<()> {
        let shared_comps = st.shared.components.read().unwrap();
        CentralPanel::default()
            .show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    st.texture_core("test")?
                        .show_size(ui, Vec2::new(24.0, 24.0));
                    if ui.button(st.text_core("ui_main")?).clicked() {
                        self.current_panel = 0;
                    }
                    if ui.button(st.text_core("ui_productions")?).clicked() {
                        self.current_panel = 1;
                    }
                    if ui.button(st.text_core("ui_debug")?).clicked() {
                        self.current_panel = 2;
                    }
                    Ok(())
                })
                .inner?;
                let mut sim_guard = st.shared.sim.lock().unwrap();
                let sim = sim_guard.as_mut().unwrap();
                match self.current_panel {
                    0 => {
                        let mut args = FluentArgs::new();
                        args.set(
                            "population",
                            sim.depot
                                .get(&st.shared.human_id)
                                .map(Clone::clone)
                                .unwrap_or_default()
                                .to_string(),
                        );
                        ui.label(st.text_core_fmt("ui_main_population", &args)?);
                        for (&id, value) in sim.depot.iter() {
                            if id != st.shared.human_id {
                                let res = shared_comps.config(id)?;
                                ui.label(format!("{} : {value}", st.text(&res.name)?));
                            }
                        }
                    }
                    1 => {
                        if ui.button(st.text_core("ui_productions_create")?).clicked() {
                            self.production_builder.open();
                        }
                        for production in sim.productions.iter() {
                            draw_production(st, &shared_comps, ui, production)?;
                        }
                    }
                    2 => {
                        ui.text_edit_singleline(&mut self.spawn_resource_name);
                        ui.text_edit_singleline(&mut self.spawn_resource_value);
                        if ui
                            .button(st.text_core("ui_debug_spawn-resources")?)
                            .clicked()
                        {
                            sim.depot.cor_put(
                                &shared_comps
                                    .core()?
                                    .configs
                                    .id_from_raw(self.spawn_resource_name.as_str())
                                    .unwrap()
                                    .in_core(),
                                self.spawn_resource_value.parse().unwrap(),
                            );
                        }
                    }
                    _ => (),
                }
                Ok(())
            })
            .inner?;

        self.production_builder.draw(st, &shared_comps, egui_ctx)?;

        Ok(())
    }
}
