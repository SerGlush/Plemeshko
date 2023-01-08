pub struct App {}

impl App {
    pub fn new() -> Self {
        App {}
    }

    pub fn gui(&mut self, context: &egui::Context) {
        egui::Window::new("Such window").show(context, |ui| {
            ui.label("Very label");
        });
    }
}
