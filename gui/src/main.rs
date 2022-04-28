use eframe::{egui::*, epi};
use std::{path::*, sync::mpsc::*};

mod keyboard_control;
use keyboard_control::*;

fn main() {
    simple_logger::SimpleLogger::new()
        .env()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let app = App::default();
    eframe::run_native(Box::new(app), eframe::NativeOptions::default());
}

struct App {
    choose_file: Option<state::ChooseFile>,
    keyboard_control: KeyboardControl<Action>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            choose_file: None,
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

    fn update(&mut self, ctx: &Context, _frame: &epi::Frame) {
        if ctx.memory().focus() == None {
            self.handle_actions(ctx);
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");

            if let Some(choose_file) = &mut self.choose_file {
                match state::ui_choose_file(ui, choose_file) {
                    None => {}
                    Some(state::FileResult::Close) => {
                        self.choose_file = None;
                    }
                    Some(state::FileResult::Selected(path)) => {
                        log::info!("{:?}", path);
                        self.choose_file = None;
                    }
                }
            }
        });
    }
}

impl App {
    fn handle_actions(&mut self, ctx: &Context) {
        for event in &ctx.input().events {
            match event {
                Event::Key {
                    pressed,
                    key,
                    modifiers,
                } => {
                    if *pressed {
                        self.keyboard_control.on_key(*key, *modifiers);
                    }
                }
                Event::Text(s) => {
                    self.keyboard_control.on_text(s);
                }
                _ => {}
            }
        }

        while let Some(a) = self.keyboard_control.take() {
            match a {
                Action::BeginOpenFile => {
                    let choose_file = state::ChooseFile::begin(ctx);
                    self.choose_file = Some(choose_file);
                }
                unhandled => {
                    log::info!("Unhandled action: {:?}", unhandled);
                }
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

mod state {
    use super::*;

    pub struct ChooseFile {
        search: String,
        files: Vec<PathBuf>,
        file_rx: Receiver<PathBuf>,
        selected: usize,
    }

    impl ChooseFile {
        pub fn begin(ctx: &Context) -> Self {
            let (tx, rx) = channel();

            let ctx = ctx.clone();
            std::thread::spawn(move || {
                ignore::Walk::new(".")
                    .filter_map(|e| e.ok())
                    .map(|e| e.into_path())
                    .filter(|p| p.is_file())
                    .for_each(|p| {
                        let _ = tx.send(p);
                        ctx.request_repaint();
                    });
            });

            ChooseFile {
                search: String::new(),
                files: Vec::new(),
                file_rx: rx,
                selected: 0,
            }
        }
    }

    pub enum FileResult {
        Close,
        Selected(PathBuf),
    }

    pub fn ui_choose_file(ui: &mut Ui, choose_file: &mut ChooseFile) -> Option<FileResult> {
        use fuzzy_matcher::FuzzyMatcher;
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

        let files = choose_file
            .files
            .iter()
            .filter(|p| {
                matcher
                    .fuzzy_match(&p.to_string_lossy(), &choose_file.search)
                    .is_some()
            })
            .take(4)
            .collect::<Vec<_>>();

        if ui.ctx().input().key_pressed(Key::ArrowDown) {
            choose_file.selected += 1;
        }
        if ui.ctx().input().key_pressed(Key::ArrowUp) {
            choose_file.selected = choose_file.selected.saturating_sub(1);
        }
        choose_file.selected = choose_file.selected.clamp(0, files.len().saturating_sub(1));

        let text_input = ui.text_edit_singleline(&mut choose_file.search);
        if text_input.lost_focus() {
            if ui.ctx().input().key_pressed(Key::Enter) {
                return Some(FileResult::Selected(
                    files[choose_file.selected].to_path_buf(),
                ));
            }
            return Some(FileResult::Close);
        }
        text_input.request_focus();

        for (i, file) in files.iter().enumerate() {
            let file = file.to_string_lossy();
            let file = file.as_ref();
            if choose_file.selected == i {
                ui.colored_label(Color32::BLACK, file);
            } else {
                ui.label(file);
            }
        }

        loop {
            match choose_file.file_rx.try_recv() {
                Ok(f) => choose_file.files.push(f),
                Err(_) => break,
            }
        }

        None
    }
}
