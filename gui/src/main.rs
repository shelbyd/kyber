use eframe::{epi, egui};

fn main() {
    let app = App::default();
    eframe::run_native(Box::new(app), eframe::NativeOptions::default());
}

#[derive(Default)]
struct App {}

impl epi::App for App {
    fn name(&self) -> &str {
        "Kyber"
    }

    fn update(&mut self, _ctx: &egui::Context, _frame: &epi::Frame) {
    }
}
