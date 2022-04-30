mod file;
pub use file::*;

use crate::Direction;

pub trait Screen {
    fn ui(&mut self, ui: &mut eframe::egui::Ui);

    fn move_cursor(&mut self, _direction: Direction) {}
}

#[derive(Default)]
pub struct Home {}

impl Screen for Home {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.heading("Hello World");
    }
}
