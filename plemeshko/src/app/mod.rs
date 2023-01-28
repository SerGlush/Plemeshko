use std::collections::HashMap;

use crate::{
    sim::{
        config::{
            method::SelectedMethod,
            method_group::MethodGroup,
            resource::ResourceId,
            transport::{Transport, TransportId},
            transport_group::TransportGroupId,
        },
        erection::Erection,
    },
    state::{components::SharedComponents, config::ConfigIndexerMap, AppState},
    util::cor::Cor,
};
use anyhow::{Ok, Result};
use egui::*;
use fluent::FluentArgs;

pub struct ErectionBuilder {
    window_is_open: bool,

    erection_name: String,
    erection_transport: HashMap<TransportGroupId, (TransportId, bool)>,
    erection_methods: Vec<SelectedMethod>,
}

impl ErectionBuilder {
    pub fn new() -> ErectionBuilder {
        ErectionBuilder {
            erection_name: "input name".to_string(),
            erection_transport: HashMap::<TransportGroupId, (TransportId, bool)>::new(),
            erection_methods: Vec::<SelectedMethod>::new(),
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
            Window::new(st.get_text_core("ui_erection_builder_title")?)
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
            ui.text_edit_singleline(&mut self.erection_name);

            self.erection_transport
                .values_mut()
                .for_each(|value| value.1 = false);

            for selected_method in &self.erection_methods {
                for selected_setting in &selected_method.settings {
                    let setting_group = shared_comps.get_config(selected_setting.group)?;
                    let mut check_resource_group = |resource_id: &ResourceId| {
                        let resource = shared_comps.get_config(*resource_id)?;

                        let mut new_group_check: bool = true;
                        for (key, value) in self.erection_transport.iter_mut() {
                            if *key == resource.transport_group {
                                value.1 = true;
                                new_group_check = false;
                            }
                        }

                        if new_group_check {
                            //let transport = shared_comps.get_config()?;
                            //self.erection_transport.insert(resource.transport_group, v);
                        }

                        Ok(())
                    };

                    let setting = setting_group.setting(shared_comps, selected_setting.index)?;
                    setting
                        .resource_io
                        .input
                        .keys()
                        .try_for_each(&mut check_resource_group)?;
                    setting
                        .resource_io
                        .output
                        .keys()
                        .try_for_each(&mut check_resource_group)?;
                }
            }

            Ok(())
        });

        for method in &mut self.erection_methods {
            ui.horizontal(|ui| {
                let method_name = st.get_text(&shared_comps.get_config(method.id)?.name)?;
                ui.label(method_name.as_ref());
                for selected_setting in &mut method.settings {
                    let setting_group = shared_comps.get_config(selected_setting.group)?;
                    let setting = setting_group.setting(shared_comps, selected_setting.index)?;
                    let setting_name = st.get_text(&setting.name)?;
                    ComboBox::from_id_source(setting_name.as_ref())
                        .width(200.0)
                        .selected_text(setting_name)
                        .show_index(
                            ui,
                            &mut selected_setting.index,
                            setting_group.settings.len(),
                            |index| match setting_group
                                .setting(shared_comps, index)
                                .and_then(|setting| st.get_text(&setting.name))
                            {
                                Result::Ok(setting_group_name) => setting_group_name.into_owned(),
                                Err(err) => err.to_string(),
                            },
                        );
                }
                Ok(())
            });
        }

        ui.menu_button(st.get_text_core("ui_erection_builder_add_method")?, |ui| {
            for method_group in shared_comps.iter_configs::<MethodGroup>() {
                let method_group = method_group?.1;
                ui.menu_button(st.get_text(&method_group.name)?, |ui| {
                    for &method_id in &method_group.variants {
                        let method = shared_comps.get_config(method_id)?;
                        if ui.button(st.get_text(&method.name)?).clicked() {
                            let selected_method =
                                SelectedMethod::new(shared_comps, method_id, None)?;
                            self.erection_methods.push(selected_method.clone());
                        }
                    }
                    Ok(())
                });
            }
            Ok(())
        });

        if ui
            .button(st.get_text_core("ui_erection_builder_create_erection")?)
            .clicked()
        {
            todo!();
        }
        Ok(())
    }

    fn get_filtered_transports(
        shared_comps: &SharedComponents,
        group_id_filter: TransportGroupId,
    ) -> Result<Vec<&Transport>> {
        let mut filtered_transports = Vec::new();
        for transport in shared_comps.iter_configs::<Transport>() {
            let transport = transport?.1;
            if transport.group == group_id_filter {
                filtered_transports.push(transport);
            }
        }
        Ok(filtered_transports)
    }
}

pub struct App {
    current_panel: i64,
    spawn_resource_name: String,
    spawn_resource_value: String,
    erection_builder: ErectionBuilder,
}

pub fn draw_erection(
    st: &AppState,
    shared_comps: &SharedComponents,
    ui: &mut Ui,
    erection: &Erection,
) -> anyhow::Result<()> {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", erection.name()));
        for &transport_id in erection.transport().values() {
            let transport = shared_comps.get_config(transport_id)?;
            let transport_group = shared_comps.get_config(transport.group)?;
            ui.label(st.get_text(&transport.name)?)
                .on_hover_text(format!(
                    "Transport Group: {}\nTransport Capacity: {}",
                    st.get_text(&transport_group.name)?,
                    transport.capacity
                ));
        }
        Ok(())
    })
    .inner?;
    for selected_method in erection.methods().iter() {
        let method = shared_comps.get_config(selected_method.id)?;
        ui.horizontal(|ui| {
            ui.label(st.get_text(&method.name)?);
            for setting in selected_method.settings.iter() {
                let setting_group = shared_comps.get_config(setting.group)?;
                ui.label(st.get_text(&setting_group.setting(shared_comps, setting.index)?.name)?);
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
            erection_builder: ErectionBuilder::new(),
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
                    st.get_texture_core("test")?
                        .show_size(ui, Vec2::new(24.0, 24.0));
                    if ui.button(st.get_text_core("ui_main")?).clicked() {
                        self.current_panel = 0;
                    }
                    if ui.button(st.get_text_core("ui_erections")?).clicked() {
                        self.current_panel = 1;
                    }
                    if ui.button(st.get_text_core("ui_debug")?).clicked() {
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
                        ui.label(st.fmt_text_core("ui_main_population", &args)?);
                        for (&id, value) in sim.depot.iter() {
                            if id != st.shared.human_id {
                                let res = shared_comps.get_config(id)?;
                                ui.label(format!("{} : {value}", st.get_text(&res.name)?));
                            }
                        }
                    }
                    1 => {
                        if ui
                            .button(st.get_text_core("ui_erections_create")?)
                            .clicked()
                        {
                            self.erection_builder.open();
                        }
                        for erection in sim.erections.iter() {
                            draw_erection(st, &shared_comps, ui, erection)?;
                        }
                    }
                    2 => {
                        ui.text_edit_singleline(&mut self.spawn_resource_name);
                        ui.text_edit_singleline(&mut self.spawn_resource_value);
                        if ui
                            .button(st.get_text_core("ui_debug_spawn-resources")?)
                            .clicked()
                        {
                            sim.depot.cor_put(
                                &shared_comps
                                    .get_core()?
                                    .configs
                                    .get_id_from_raw(self.spawn_resource_name.as_str())
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

        self.erection_builder.draw(st, &shared_comps, egui_ctx)?;

        Ok(())
    }
}
