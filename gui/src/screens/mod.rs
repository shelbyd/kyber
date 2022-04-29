mod file;
pub use file::*;

pub trait Screen {
    fn ui(&mut self, ui: &mut eframe::egui::Ui);
}

#[derive(Default)]
pub struct Home {}

impl Screen for Home {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.heading("Hello World");
    }
}
