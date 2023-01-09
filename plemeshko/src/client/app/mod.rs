use crate::server::Sim;

pub struct App {}

impl App {
    pub fn new() -> Self {
        App {}
    }

    pub fn update(&mut self, sim: &mut Sim) {}

    pub fn gui(&mut self, context: &egui::Context, sim: &mut Sim) {
        egui::Window::new("Such window").show(context, |ui| {
            ui.label("Very label");
        });
    }
}
