use std::borrow::Cow;

use anyhow::{Ok, Result};
use egui::{vec2, Button, Color32, RichText};
use tap::Conv;

use crate::{
    app::{
        env::Env,
        util::{
            draw_icon, draw_resource_io_tt, draw_resource_io_tt_lazy, on_using_modifiers,
            ConfigIteratorExt,
        },
        widgets::{PersistentWindow, Tab, Widget},
    },
    sim::{config::resource::ResourceMap, production::Production},
    state::{
        components::SharedComponents,
        has::{HasSimMutex, HasTexts},
        AppState,
    },
    util::cor::Cor,
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
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    production_index: usize,
    productions: &mut Vec<Production>,
    depot: &mut ResourceMap,
) -> Result<bool> {
    ui.separator();
    let removed = ui
        .horizontal(|ui| {
            let production = &mut productions[production_index];

            let prodname_response = ui.strong(format!("{}:", production.name()));
            let prodname_tt_shift = prodname_response.ctx.input().modifiers.shift_only();
            let prodname_tt_cmd = prodname_response.ctx.input().modifiers.command_only();
            draw_resource_io_tt_lazy(app_st, shared_comps, ctx, prodname_response, || {
                if prodname_tt_shift {
                    return Cow::Borrowed(production.single_io());
                }
                if prodname_tt_cmd {
                    let mut io = production.single_io().clone();
                    for (_, amount) in io.input.iter_mut() {
                        amount.0 *= production.active().conv::<i64>();
                    }
                    for (_, amount) in io.output.iter_mut() {
                        amount.0 *= production.active().conv::<i64>();
                    }
                    return Cow::Owned(io);
                }
                Cow::Borrowed(production.last_io())
            });

            let inactive = production.count() - production.active();
            let can_grow = depot.cor_has_all_times(
                production.cost(),
                if inactive < 100 {
                    100 - inactive as i64
                } else {
                    0
                },
            ) + inactive as i64;
            let grow: u32 = match (
                ctx.input().modifiers.command_only(),
                ctx.input().modifiers.shift_only(),
            ) {
                (true, false) => 100,
                (false, true) => 10,
                _ => 1,
            };
            let (color, enabled) = if can_grow >= grow as i64 {
                (Color32::WHITE, true)
            } else {
                (Color32::from_rgb(255, 200, 200), false)
            };
            let grow_response =
                ui.add(Button::new(RichText::new("+").color(color)).min_size(vec2(16.0, 16.0)));
            if enabled && grow_response.clicked() {
                let new_active = production.active() + grow;
                if new_active > production.count() {
                    production.set_count(new_active);
                }
                production.set_active(new_active);
                if inactive < grow {
                    depot.cor_sub_all_times_unchecked(production.cost(), (grow - inactive) as i64);
                }
            }
            grow_response.on_hover_ui(|ui| {
                for (&id, &amount) in production.cost() {
                    ui.horizontal(|ui| {
                        let icon = &shared_comps.config(id).unwrap().info.icon;
                        let tint = if depot.cor_has_times(&id, amount) >= grow as i64 {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(255, 200, 200)
                        };
                        draw_icon(app_st, ctx, ui, icon, vec2(24., 24.), |i| i.tint(tint)).unwrap();
                        ui.label(amount.to_string());
                    });
                }
            });

            ui.label(format!(
                "{}/{}/{}",
                production.last_activated(),
                production.active(),
                production.count()
            ));

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
                        "{}: {}\n{}: {}",
                        app_st.text_core("ui_generic_transport-group")?,
                        app_st.text(&transport_group.name)?,
                        app_st.text_core("ui_generic_transport-capacity")?,
                        transport.capacity
                    ));
            }
            Ok(false)
        })
        .inner?;

    if removed {
        return Ok(true);
    }

    for selected_method in productions[production_index].methods() {
        let method = shared_comps.config(selected_method.id)?;
        ui.horizontal(|ui| {
            ui.label(app_st.text(&method.info.name)?);
            for setting in selected_method.settings.configs(shared_comps) {
                let response = ui.label(app_st.text(&setting.name)?);
                draw_resource_io_tt(app_st, shared_comps, ctx, response, &setting.resource_io);
            }
            Ok(())
        })
        .inner?;
    }
    Ok(false)
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
        let ctx = env.get::<egui::Context>().unwrap();
        let shared_comps = env.shared_components();
        let mut sim_guard = app_st.lock_sim();
        let sim = sim_guard.as_mut().unwrap();
        let mut production_index = 0;
        while production_index < sim.productions.len() {
            let removed = ui_production(
                app_st,
                shared_comps,
                ctx,
                ui,
                production_index,
                &mut sim.productions,
                &mut sim.depot,
            )?;
            if !removed {
                production_index += 1;
            }
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
