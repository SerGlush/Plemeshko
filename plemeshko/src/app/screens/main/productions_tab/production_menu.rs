use std::collections::HashMap;

use anyhow::{Ok, Result};
use egui::{vec2, ComboBox, Ui};

use crate::{
    app::{
        env::Env,
        util::{draw_icon_btn_with_tooltip, draw_iter_indexed, ConfigIteratorExt},
        widgets::{PersistentWindowContent, Widget, WindowCloseEvent},
    },
    sim::{
        config::{
            production_method::{FixedProductionMethod, ProductionMethodId},
            production_method_group::ProductionMethodGroup,
            resource::Resource,
            transport_group::TransportGroupId,
            transport_method::TransportMethodId,
        },
        production::Production,
        Sim,
    },
    state::{
        components::SharedComponents,
        has::{HasSimMutex, HasTexts},
        AppState,
    },
};

#[derive(Default, Clone)]
pub struct ProductionBuilder {
    name: String,
    transport_methods: HashMap<TransportGroupId, (TransportMethodId, bool)>,
    production_methods: Vec<FixedProductionMethod>,
}

impl ProductionBuilder {
    pub fn new() -> Self {
        ProductionBuilder::default()
    }

    pub fn finish(&self, shared_comps: &SharedComponents) -> Result<Production> {
        Production::new(
            shared_comps,
            self.name.clone(),
            self.production_methods.clone(),
            self.transport_methods
                .iter()
                .map(|(&key, &(value, _))| (key, value))
                .collect(),
        )
    }

    fn draw_method_select(
        &mut self,
        app_st: &AppState,
        shared_comps: &SharedComponents,
        ctx: &egui::Context,
        ui: &mut Ui,
        sim: &Sim,
        mut variants: impl Iterator<Item = ProductionMethodId>,
    ) -> Result<()> {
        loop {
            let mut reached_end = false;
            let mut method_row_current = 0;
            const METHOD_ROW_SIZE: u32 = 8;
            ui.horizontal(|ui| {
                while method_row_current < METHOD_ROW_SIZE {
                    let Some(method_id) = variants.next() else {
                    reached_end = true;
                    break;
                };
                    if !sim.research.is_production_unlocked(method_id) {
                        continue;
                    }
                    let method = shared_comps.config(method_id)?;
                    draw_icon_btn_with_tooltip(
                        app_st,
                        ctx,
                        ui,
                        &method.info,
                        vec2(24., 24.),
                        |i| i,
                        |_| (),
                        || {
                            let selected_method =
                                FixedProductionMethod::new(shared_comps, method_id, None)?;
                            self.production_methods.push(selected_method.clone());
                            Ok(())
                        },
                    )?;
                    method_row_current += 1;
                }
                Ok(())
            })
            .inner?;
            if reached_end {
                return Ok(());
            }
        }
    }
}

impl Widget for ProductionBuilder {
    type Response = ();

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut egui::Ui) -> Result<Self::Response> {
        let app_st = env.app_state();
        let shared_comps = env.shared_components();
        let ctx = env.get::<egui::Context>().unwrap();
        let mut sim_guard = app_st.lock_sim();
        let sim = sim_guard.as_mut().unwrap();
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.name)
                    .hint_text(app_st.text_core("ui_main_productions_builder_name")?),
            );

            self.transport_methods.values_mut().try_for_each(|value| {
                let selected_transport = shared_comps.config(value.0)?;
                value.1 = false;

                let selected_transport_name = app_st.text(&selected_transport.info.name)?;
                ComboBox::from_id_source(selected_transport_name.as_ref())
                    .selected_text(selected_transport_name.as_ref())
                    .show_ui(ui, |ui| {
                        let transport_group = shared_comps.config(selected_transport.group)?;
                        for (transport_id, transport) in transport_group
                            .transports
                            .configs_with_ids(shared_comps)
                            .filter(|(id, _)| sim.research.is_transport_unlocked(*id))
                        {
                            let transport_name = app_st.text(&transport.info.name)?;

                            if ui.button(transport_name).clicked() {
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
                        for (key, value) in self.transport_methods.iter_mut() {
                            if *key == resource.transport_group {
                                value.1 = true;
                                new_group_check = false;
                            }
                        }

                        if new_group_check {
                            let transport_id = &shared_comps
                                .config(resource.transport_group)?
                                .transports
                                .configs_with_ids(shared_comps)
                                .find(|(_, tr)| {
                                    // note: `ui_priority` must be 0 only for unlocked transoport methods
                                    tr.ui_priority == 0 && tr.group == resource.transport_group
                                })
                                .unwrap()
                                .0;
                            self.transport_methods
                                .insert(resource.transport_group, (*transport_id, true));
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
                let method_name = app_st.text(&shared_comps.config(method.id)?.info.name)?;
                ui.label(method_name.as_ref());
                for selected_setting_id in &mut method.settings {
                    let selected_setting = shared_comps.config(*selected_setting_id)?;
                    let setting_group = shared_comps.config(selected_setting.group)?;
                    let selected_setting_name = app_st.text(&selected_setting.name)?;
                    ComboBox::from_id_source(&selected_setting_name)
                        .width(200.0)
                        .selected_text(selected_setting_name)
                        .show_ui(ui, |ui| {
                            for &setting_id in &setting_group.settings {
                                let setting = shared_comps.config(setting_id)?;
                                if ui
                                    .selectable_label(false, app_st.text(&setting.name)?)
                                    .clicked()
                                {
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

        ui.menu_button(
            app_st.text_core("ui_main_productions_builder_add-production-method")?,
            |ui| {
                for method_group in shared_comps.iter_configs::<ProductionMethodGroup>() {
                    let method_group = method_group?.1;
                    ui.menu_button(app_st.text(&method_group.name)?, |ui| {
                        self.draw_method_select(
                            app_st,
                            shared_comps,
                            ctx,
                            ui,
                            sim,
                            method_group.variants.iter().copied(),
                        )
                    });
                }
                Ok(())
            },
        );

        if ui
            .button(app_st.text_core("ui_main_productions_builder_finish")?)
            .clicked()
        {
            sim.productions.push(self.finish(shared_comps)?);
            env.get::<WindowCloseEvent<ProductionBuilder>>()
                .map(WindowCloseEvent::emit);
            *self = Default::default();
        }
        Ok(())
    }
}

impl PersistentWindowContent for ProductionBuilder {
    fn title(&self, env: &Env<'_>) -> Result<egui::WidgetText> {
        env.app_state()
            .text_core("ui_main_productions_builder_window-title")
            .map(Into::into)
    }
}
