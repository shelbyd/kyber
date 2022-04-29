use eframe::{egui::*, epi};
use std::{cmp::*, collections::*, path::*, sync::mpsc::*};

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
                let result = Window::new("Choose file")
                    .anchor(Align2::CENTER_TOP, [0., 10.])
                    .show(ctx, |ui| choose_file.ui(ui))
                    .and_then(|r| r.inner?);

                match result {
                    None => {}
                    Some(state::FileResult::Close) => {
                        self.choose_file = None;
                    }
                    Some(state::FileResult::Selected(path)) => {
                        log::info!("Would open file: {:?}", path);
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
                    self.choose_file = Some(state::ChooseFile::begin());
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
        pub fn begin() -> Self {
            let (tx, rx) = channel();

            std::thread::spawn(move || {
                let result = ignore::Walk::new(".")
                    .filter_map(|e| {
                        let path = e.ok()?.into_path();
                        let without_leading = path.strip_prefix("./").ok()?;
                        Some(without_leading.to_path_buf())
                    })
                    .filter(|p| p.is_file())
                    .try_for_each(|p| tx.send(p));
                if let Err(e) = result {
                    log::error!("{}", e);
                }
            });

            ChooseFile {
                search: String::new(),
                files: Vec::new(),
                file_rx: rx,
                selected: 0,
            }
        }

        pub fn ui(&mut self, ui: &mut Ui) -> Option<FileResult> {
            let files = SortedByKey::new(
                self.files.iter().filter_map(|p| {
                    if self.search.is_empty() {
                        return Some((p, 0));
                    }

                    let match_ = sublime_fuzzy::best_match(&self.search, &p.to_string_lossy())?;
                    Some((p, match_.score()))
                }),
                |(_, i)| *i,
            )
            .map(|(p, _)| p)
            .take(4)
            .collect::<Vec<_>>();

            if ui.ctx().input().key_pressed(Key::ArrowDown) {
                self.selected += 1;
            }
            if ui.ctx().input().key_pressed(Key::ArrowUp) {
                self.selected = self.selected.saturating_sub(1);
            }
            self.selected = self.selected.clamp(0, files.len().saturating_sub(1));

            let text_input = ui.text_edit_singleline(&mut self.search);
            if text_input.lost_focus() {
                if ui.ctx().input().key_pressed(Key::Enter) {
                    return Some(FileResult::Selected(files[self.selected].to_path_buf()));
                }
                return Some(FileResult::Close);
            }
            text_input.request_focus();

            for (i, file) in files.iter().enumerate() {
                let file = file.to_string_lossy();
                let file = file.as_ref();
                if self.selected == i {
                    ui.colored_label(Color32::BLACK, file);
                } else {
                    ui.label(file);
                }
            }

            loop {
                match self.file_rx.try_recv() {
                    Ok(f) => self.files.push(f),
                    Err(_) => break,
                }
            }

            None
        }
    }

    pub enum FileResult {
        Close,
        Selected(PathBuf),
    }
}

struct SortedByKey<T, K> {
    heap: BinaryHeap<Reverse<Keyed<T, K>>>,
}

impl<T, K> SortedByKey<T, K>
where
    K: Ord,
{
    fn new(iter: impl IntoIterator<Item = T>, mut key_extractor: impl FnMut(&T) -> K) -> Self {
        let heap = iter
            .into_iter()
            .map(|t| {
                Reverse(Keyed {
                    key: key_extractor(&t),
                    value: t,
                })
            })
            .collect();
        SortedByKey { heap }
    }
}

impl<T, K> Iterator for SortedByKey<T, K>
where
    K: Ord,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.heap.pop().map(|keyed| keyed.0.value)
    }
}

struct Keyed<T, K> {
    value: T,
    key: K,
}

impl<T, K> PartialEq for Keyed<T, K>
where
    K: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T, K> Eq for Keyed<T, K> where K: Eq {}

impl<T, K> PartialOrd for Keyed<T, K>
where
    K: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<T, K> Ord for Keyed<T, K>
where
    K: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}
