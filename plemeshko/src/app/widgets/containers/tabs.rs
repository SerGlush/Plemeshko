use anyhow::Result;
use egui::{plot::Orientation, Ui, WidgetText};

use crate::app::{
    env::Env,
    widgets::{Widget, WidgetMap},
};

pub trait Tab: Widget {
    fn header(&self, env: &Env<'_>) -> Result<WidgetText>;
}

impl<W: Tab, R, F: FnMut(W::Response) -> Result<R>> Tab for WidgetMap<W, R, F> {
    fn header(&self, env: &Env<'_>) -> Result<WidgetText> {
        self.0.header(env)
    }
}

pub struct Tabs<R> {
    tabs: Vec<Box<dyn Tab<Response = R>>>,
    selected: usize,
    orientation: Orientation,
}

impl<R> Tabs<R> {
    /// * `orientation` - Orientation (horizontal or vertical) of the header array.
    pub fn new(orientation: Orientation) -> Self {
        Tabs {
            tabs: Vec::new(),
            selected: 0,
            orientation,
        }
    }

    pub fn push(&mut self, tab: impl Tab<Response = R> + 'static) {
        self.tabs.push(Box::new(tab))
    }
}

impl<R> Widget for Tabs<R> {
    type Response = R;

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut egui::Ui) -> Result<R> {
        assert!(!self.tabs.is_empty(), "Empty `Tabs` can't be shown.");
        let ui_headers = |ui: &mut Ui| -> Result<()> {
            for (tab_index, tab) in self.tabs.iter().enumerate() {
                if ui
                    .selectable_label(tab_index == self.selected, tab.header(env)?)
                    .clicked()
                {
                    self.selected = tab_index;
                }
            }
            Ok(())
        };
        match self.orientation {
            Orientation::Horizontal => {
                ui.horizontal(ui_headers).inner?;
                ui.separator();
            }
            Orientation::Vertical => {
                ui.vertical(ui_headers).inner?;
                ui.separator();
            }
        }
        self.tabs[self.selected].ui(env, ui)
    }
}
