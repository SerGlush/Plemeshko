use std::{collections::HashMap, convert::identity};

use anyhow::{Ok, Result};
use egui::{vec2, Color32, ComboBox, Ui};

use crate::{
    app::{
        env::Env,
        util::{
            draw_icon_btn_with_tooltip, draw_icon_with_tooltip, draw_iter_indexed,
            draw_resource_io, draw_resource_map_labeled, ConfigIteratorExt,
        },
        widgets::{PersistentWindowContent, Widget, WindowCloseEvent},
    },
    sim::{
        config::{
            production_method::{FixedProductionMethod, ProductionMethodId},
            production_method_group::ProductionMethodGroup,
            resource::{Resource, ResourceIo, ResourceMap},
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
    util::cor::Cor,
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

    pub fn ready(&self) -> bool {
        !self.name.is_empty() && !self.production_methods.is_empty()
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
                            self.production_methods.push(selected_method);
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

        let mut unwanted_method: Option<ProductionMethodId> = None;

        draw_iter_indexed(ui, self.production_methods.iter_mut(), |ui, method| {
            ui.horizontal(|ui| {
                let method_name = app_st.text(&shared_comps.config(method.id)?.info.name)?;
                ui.label(method_name.as_ref());
                for selected_setting_id in &mut method.settings {
                    let selected_setting = shared_comps.config(*selected_setting_id)?;
                    let setting_group = shared_comps.config(selected_setting.group)?;
                    let selected_setting_name = app_st.text(&selected_setting.name)?;
                    let cb_response = ComboBox::from_id_source(&selected_setting_name)
                        .width(200.0)
                        .selected_text(selected_setting_name)
                        .show_ui(ui, |ui| {
                            for &setting_id in &setting_group.settings {
                                let setting = shared_comps.config(setting_id)?;
                                let response =
                                    ui.selectable_label(false, app_st.text(&setting.name)?);
                                if response.clicked() {
                                    *selected_setting_id = setting_id;
                                }
                                response.on_hover_ui(|ui| {
                                    draw_resource_io(
                                        app_st,
                                        shared_comps,
                                        ctx,
                                        ui,
                                        &setting.resource_io,
                                    )
                                    .unwrap();
                                    draw_resource_map_labeled(
                                        app_st,
                                        shared_comps,
                                        ctx,
                                        ui,
                                        &setting.cost,
                                        app_st.text_core("ui_generic_cost").unwrap(),
                                        true,
                                    )
                                    .unwrap();
                                });
                            }
                            Ok(())
                        });
                    cb_response.inner.transpose()?;
                    cb_response.response.on_hover_ui(|ui| {
                        draw_resource_io(
                            app_st,
                            shared_comps,
                            ctx,
                            ui,
                            &selected_setting.resource_io,
                        )
                        .unwrap();
                        draw_resource_map_labeled(
                            app_st,
                            shared_comps,
                            ctx,
                            ui,
                            &selected_setting.cost,
                            app_st.text_core("ui_generic_cost").unwrap(),
                            true,
                        )
                        .unwrap();
                    });
                }

                if ui.button("X").clicked() {
                    unwanted_method = Some(method.id);
                };

                Ok(())
            })
            .inner
        })?;

        if let Some(needed_method) = unwanted_method {
            self.production_methods.remove(
                self.production_methods
                    .iter()
                    .position(|method| method.id == needed_method)
                    .unwrap(),
            );
        }

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

        let mut total_cost = ResourceMap::new();
        let mut total_io = ResourceIo::new();
        for production_method in &self.production_methods {
            production_method.accumulate(shared_comps, &mut total_cost, &mut total_io)?;
        }
        let mut total_input: Vec<_> = total_io.input.into_iter().collect();
        total_input.sort_by_key(|(id, _)| *id);
        let mut total_output: Vec<_> = total_io.output.into_iter().collect();
        total_output.sort_by_key(|(id, _)| *id);

        let mut enough_resources = true;
        ui.separator();
        ui.horizontal(|ui| {
            let half_width = ui.available_width() / 2.;
            ui.vertical(|ui| {
                if !total_input.is_empty() {
                    ui.strong(app_st.text_core("ui_generic_input")?);
                    ui.indent("total input", |ui| {
                        for (id, amount) in total_input {
                            let info = &shared_comps.config(id)?.info;
                            ui.horizontal(|ui| {
                                draw_icon_with_tooltip(
                                    app_st,
                                    ctx,
                                    ui,
                                    info,
                                    vec2(32., 32.),
                                    identity,
                                    |_| (),
                                )?;
                                ui.label(amount.to_string());
                                Ok(())
                            })
                            .inner?;
                        }
                        Ok(())
                    })
                    .inner?;
                }
                if !total_output.is_empty() {
                    ui.strong(app_st.text_core("ui_generic_output")?);
                    ui.indent("total output", |ui| {
                        for (id, amount) in total_output {
                            let info = &shared_comps.config(id)?.info;
                            ui.horizontal(|ui| {
                                draw_icon_with_tooltip(
                                    app_st,
                                    ctx,
                                    ui,
                                    info,
                                    vec2(32., 32.),
                                    identity,
                                    |_| (),
                                )?;
                                ui.label(amount.to_string());
                                Ok(())
                            })
                            .inner?;
                        }
                        Ok(())
                    })
                    .inner?;
                }
                Ok(())
            })
            .inner?;
            ui.allocate_exact_size(
                vec2(ui.available_width() - half_width, 1.),
                egui::Sense::hover(),
            );
            ui.vertical(|ui| {
                if !total_cost.is_empty() {
                    ui.strong(app_st.text_core("ui_generic_cost")?);
                    ui.indent("total cost", |ui| {
                        for (&id, &cost_amount) in &total_cost {
                            let resource = shared_comps.config(id)?;
                            let stored_amount = sim.depot.get(&id).copied().unwrap_or_default();
                            let tint = if cost_amount <= stored_amount {
                                Color32::WHITE
                            } else {
                                enough_resources = false;
                                Color32::from_rgb(240, 160, 160)
                            };
                            ui.horizontal(|ui| {
                                draw_icon_with_tooltip(
                                    app_st,
                                    ctx,
                                    ui,
                                    &resource.info,
                                    vec2(32., 32.),
                                    |i| i.tint(tint),
                                    |_| (),
                                )?;
                                ui.colored_label(tint, cost_amount.to_string());
                                Ok(())
                            })
                            .inner?;
                        }
                        Ok(())
                    })
                    .inner?;
                }
                Ok(())
            })
            .inner
        })
        .inner?;

        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    enough_resources && self.ready(),
                    egui::Button::new(app_st.text_core("ui_main_productions_builder_finish")?),
                )
                .clicked()
            {
                sim.depot.cor_sub_all_unchecked(&total_cost);
                sim.productions.push(self.finish(shared_comps)?);
                env.get::<WindowCloseEvent<ProductionBuilder>>()
                    .map(WindowCloseEvent::emit);
                *self = Default::default();
            }
            if ui.button(app_st.text_core("ui_generic_clear")?).clicked() {
                self.production_methods.clear();
            }
            Ok(())
        })
        .inner?;
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
