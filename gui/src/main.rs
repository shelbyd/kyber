use eframe::{egui, epi};

mod keyboard_control;
use keyboard_control::*;

fn main() {
    let app = App::default();
    eframe::run_native(Box::new(app), eframe::NativeOptions::default());
}

struct App {
    keyboard_control: KeyboardControl<Action>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            keyboard_control: KeyboardControl::new(maplit::hashmap! {
                "j" => Action::Cursor(Direction::Down),
                "k" => Action::Cursor(Direction::Up),
                "l" => Action::Cursor(Direction::Right),
                "h" => Action::Cursor(Direction::Left),

                "of" => Action::BeginOpenFile,
            }),
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "Kyber"
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            for event in &ctx.input().events {
                match event {
                    egui::Event::Key {
                        pressed,
                        key,
                        modifiers,
                    } => {
                        if *pressed {
                            self.keyboard_control.on_key(*key, *modifiers);
                        }
                    }
                    egui::Event::Text(s) => {
                        self.keyboard_control.on_text(s);
                    }
                    _ => {}
                }
            }

            while let Some(a) = self.keyboard_control.take() {
                self.on_action(a);
            }

            ui.heading("Hello World!");
        });
    }
}

impl App {
    fn on_action(&mut self, action: Action) {
        match action {
            unhandled => {
                unimplemented!("unhandled: {:?}", unhandled);
            }
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
enum Action {
    BeginOpenFile,
    Cursor(Direction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}
