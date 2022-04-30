#[derive(Default)]
pub struct Cursor {}

impl Cursor {
    pub fn byte_pos(&self, _text: &str) -> usize {
        log::warn!("Unimplemented Cursor#pos");
        3
    }
}
