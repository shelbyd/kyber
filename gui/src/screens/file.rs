use super::*;

use eframe::egui::{containers::*, *};
use std::path::*;

use crate::background::*;

pub struct File {
    path: PathBuf,
    contents: BackgroundJob<String>,
}

impl File {
    pub fn with_path(path: PathBuf) -> Self {
        File {
            path: path.clone(),
            contents: BackgroundJob::run(move || std::fs::read_to_string(&path).unwrap()),
        }
    }
}

impl Screen for File {
    fn ui(&mut self, ui: &mut Ui) {
        let contents = match self.contents.value() {
            Some(contents) => contents,
            None => {
                ui.heading(format!("Loading {}", self.path.to_string_lossy()));
                return;
            }
        };

        ScrollArea::both().show(ui, |ui| {
            ui.label(
                RichText::new(contents)
                    .monospace()
                    .background_color(Color32::TRANSPARENT),
            );
        });
    }
}
