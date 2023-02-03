use std::ops::Deref;

use anyhow::Result;

use crate::{
    app::{
        env::Env,
        widgets::{Tabs, Widget},
    },
    state::AppState,
};

use self::{
    debug_tab::MainScreenDebugTab, info_tab::MainScreenInfoTab,
    productions_tab::MainScreenProductionsTab,
};

mod debug_tab;
mod info_tab;
mod productions_tab;

pub struct MainScreen(Tabs<()>);

impl MainScreen {
    pub fn new() -> Self {
        let mut tabs = Tabs::new(egui::plot::Orientation::Horizontal);
        tabs.push(MainScreenInfoTab::new());
        tabs.push(MainScreenProductionsTab::new());
        tabs.push(MainScreenDebugTab::new());
        MainScreen(tabs)
    }
}

impl Widget for MainScreen {
    type Response = ();

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut egui::Ui) -> Result<Self::Response> {
        let shared_comps = env
            .get::<AppState>()
            .unwrap()
            .shared
            .components
            .read()
            .unwrap();
        env.with(shared_comps.deref(), |env| self.0.ui(env, ui))
    }
}
