use std::{
    fmt::Write,
    time::{Duration, Instant},
};

use anyhow::{Ok, Result};
use egui::{Color32, Pos2, ProgressBar, Ui, WidgetText};

use crate::{
    app::{
        env::Env,
        util::draw_icon_with_tooltip,
        widgets::{Tab, Widget},
    },
    sim::{
        config::technology::{Technology, TechnologyBonus, TechnologyId},
        Sim,
    },
    state::{
        has::{HasSimMutex, HasTexts},
        AppState,
    },
};

pub struct MainScreenResearchTab {
    tooltip_closed: Option<Instant>,
    technology_tooltip: Option<(TechnologyId, Pos2)>,
}

const TECHNOLOGY_ICON_SIZE: f32 = 64.0;

impl MainScreenResearchTab {
    pub fn new() -> Self {
        MainScreenResearchTab {
            tooltip_closed: None,
            technology_tooltip: None,
        }
    }
}

impl Tab for MainScreenResearchTab {
    fn header(&self, env: &Env<'_>) -> Result<WidgetText> {
        let app_st = env.get::<AppState>().unwrap();
        Ok(app_st.text_core("ui_main_research_header")?.into())
    }
}

impl Widget for MainScreenResearchTab {
    type Response = ();

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut egui::Ui) -> Result<()> {
        let app_st = env.app_state();
        let shared_comps = env.shared_components();
        let mut sim_guard = app_st.lock_sim();
        let sim = sim_guard.as_mut().unwrap();
        let ctx = env.get::<egui::Context>().unwrap();

        let mut current_research_text = app_st.text_core("ui_main_research_current")?.into_owned();
        if let Some((id, progress)) = sim.research.current() {
            write!(
                current_research_text,
                ": {}",
                app_st.text(&shared_comps.config(id)?.info.name)?
            )?;
            ui.horizontal(|ui| {
                ui.label(current_research_text);
                ui.add(ProgressBar::new(
                    progress as f32 / shared_comps.config(id)?.cost as f32,
                ));
                Ok(())
            })
            .inner?;
        } else {
            write!(
                current_research_text,
                ": {}",
                app_st.text_core("ui_generic_nothing")?
            )?;
            ui.label(current_research_text);
        }
        ui.separator();

        if self
            .tooltip_closed
            .is_some_and(|t| t.elapsed() > Duration::from_millis(200))
        {
            self.technology_tooltip = None;
        }

        let mut technology_hovered = false;
        let technology_row_size = (ui.available_width() / TECHNOLOGY_ICON_SIZE).floor() as u32;
        let mut technology_iter = shared_comps.iter_configs::<Technology>();
        loop {
            let mut reached_end = false;
            ui.horizontal(|ui| {
                let mut technology_row_current = 0;
                while technology_row_current < technology_row_size {
                    let Some(technology) = technology_iter.next() else {
                        reached_end = true;
                        break;
                    };
                    let (id, technology) = technology?;
                    draw_technology_icon_tip(
                        app_st,
                        ctx,
                        ui,
                        sim,
                        id,
                        technology,
                        self,
                        &mut technology_hovered,
                    )?;
                    technology_row_current += 1;
                }
                Ok(())
            })
            .inner?;
            if reached_end {
                break;
            }
        }

        if let Some((id, pos)) = self.technology_tooltip {
            let technology = shared_comps.config(id)?;
            let win_resp = egui::Window::new("tech tooltip")
                .title_bar(false)
                .default_pos(pos)
                .show(ctx, |ui| {
                    ui.label(app_st.text(&technology.info.name)?);
                    ui.colored_label(
                        ui.visuals().weak_text_color(),
                        app_st.text(&technology.info.description)?,
                    );
                    ui.separator();
                    ui.horizontal(|ui| {
                        for bonus in &technology.bonuses {
                            let info = match bonus {
                                TechnologyBonus::UnlockTransport(tr_id) => {
                                    &shared_comps.config(*tr_id)?.info
                                }
                                TechnologyBonus::UnlockProduction(pr_id) => {
                                    &shared_comps.config(*pr_id)?.info
                                }
                            };
                            draw_icon_with_tooltip(
                                app_st,
                                ctx,
                                ui,
                                info,
                                egui::vec2(24., 24.),
                                |i| i,
                                |_| (),
                            )?;
                        }
                        Ok(())
                    })
                    .inner?;
                    Ok(())
                });
            if let Some(win_resp) = win_resp {
                if !technology_hovered
                    && !win_resp.response.hovered()
                    // idk how to detect if specific window is being dragged, not much to drag there anyway
                    && !ctx.memory().is_anything_being_dragged()
                {
                    if self.tooltip_closed.is_none() {
                        self.tooltip_closed = Some(Instant::now());
                    }
                } else {
                    self.tooltip_closed = None;
                }
                win_resp.inner.transpose()?;
            }
        }

        Ok(())
    }
}

fn draw_technology_icon_tip(
    app_st: &AppState,
    ctx: &egui::Context,
    ui: &mut Ui,
    sim: &mut Sim,
    id: TechnologyId,
    technology: &Technology,
    this: &mut MainScreenResearchTab,
    tt: &mut bool,
) -> Result<()> {
    let is_researched = sim.research.is_researched(id);
    let prerequisites_satisfied = technology.prerequisites_satisfied();
    let tint = match (is_researched, prerequisites_satisfied) {
        (false, false) => Color32::from_rgb(255, 120, 120),
        (false, true) => {
            if sim
                .research
                .current()
                .is_some_and(|(cur_id, _)| cur_id == id)
            {
                Color32::from_rgb(155, 155, 255)
            } else {
                Color32::from_rgb(155, 155, 155)
            }
        }
        (true, _) => Color32::WHITE,
    };
    let mut button = egui::ImageButton::new(
        app_st
            .texture(technology.info.icon.texture)?
            .texture_id(ctx),
        egui::vec2(TECHNOLOGY_ICON_SIZE, TECHNOLOGY_ICON_SIZE),
    );
    if let Some(uv) = technology.info.icon.uv {
        button = button.uv(uv);
    }
    let response = ui.add(button.tint(tint));

    if response.hovered() {
        this.technology_tooltip = Some((id, ui.next_widget_position()));
        *tt = true;
    }

    if response.clicked() && !is_researched && prerequisites_satisfied {
        sim.research.start(id);
    }
    Ok(())
}
