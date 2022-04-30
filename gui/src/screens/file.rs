use super::*;

use eframe::egui::{containers::*, text::*, *};
use std::path::*;

use crate::{background::*, cursor::*};

pub struct File {
    path: PathBuf,
    contents: BackgroundJob<String>,
    cursor: Cursor,
}

impl File {
    pub fn with_path(path: PathBuf) -> Self {
        File {
            path: path.clone(),
            contents: BackgroundJob::run(move || std::fs::read_to_string(&path).unwrap()),
            cursor: Cursor::default(),
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

        let pos = self.cursor.byte_pos(&contents);
        let (pre, cursor, post) = (
            &contents[0..pos],
            &contents[pos..(pos + 1)],
            &contents[(pos + 1)..],
        );

        let normal_format = TextFormat {
            font_id: FontId::new(14.0, FontFamily::Monospace),
            color: Color32::LIGHT_GRAY,
            ..Default::default()
        };

        let mut layout_job = LayoutJob::default();
        layout_job.append(pre, 0.0, normal_format.clone());
        layout_job.append(
            cursor,
            0.0,
            TextFormat {
                color: Color32::DARK_GRAY,
                background: Color32::LIGHT_GRAY,
                ..normal_format.clone()
            },
        );
        layout_job.append(post, 0.0, normal_format);

        ScrollArea::both().show(ui, |ui| ui.label(layout_job));
    }
}
