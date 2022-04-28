use eframe::egui::{Key, Modifiers};
use std::collections::*;

pub struct KeyboardControl<A> {
    map: HashMap<String, A>,
    history: String,
}

impl<A: Clone> KeyboardControl<A> {
    pub fn new(map: HashMap<&str, A>) -> Self {
        Self {
            map: map.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            history: String::new(),
        }
    }

    pub fn on_key(&mut self, key: Key, _modifiers: Modifiers) {
        let as_str = format!("{:?}", key);
        if as_str.len() > 1 {
            self.history += &format!("<{}>", as_str);
        };
    }

    pub fn on_text(&mut self, text: &str) {
        self.history += text;
    }

    pub fn take(&mut self) -> Option<A> {
        if let Some((seq, a)) = self
            .map
            .iter()
            .find(|(seq, _)| self.history.starts_with(*seq))
        {
            self.history = self.history.split_off(seq.len());
            return Some(a.clone());
        }

        let could_be_in_progress = self.map.keys().any(|k| k.starts_with(&self.history));
        if could_be_in_progress {
            return None;
        }

        if self.map.is_empty() {
            self.history.clear();
            return None;
        }

        self.history.remove(0);
        self.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrecognized_is_none() {
        let mut control = KeyboardControl::<()>::new(HashMap::new());

        control.on_text("o");

        assert_eq!(control.take(), None);
    }

    #[test]
    fn single_char() {
        let mut control = KeyboardControl::new(maplit::hashmap! {
            "j" => "down",
        });

        control.on_text("j");

        assert_eq!(control.take(), Some("down"));
    }

    #[test]
    fn char_sequence() {
        let mut control = KeyboardControl::new(maplit::hashmap! {
            "of" => "open_file",
        });

        control.on_text("o");
        control.on_text("f");

        assert_eq!(control.take(), Some("open_file"));
    }

    #[test]
    fn sequence_of_actions() {
        let mut control = KeyboardControl::new(maplit::hashmap! {
            "j" => "cursor_down",
            "of" => "open_file",
        });

        control.on_text("j");
        control.on_text("o");
        control.on_text("f");

        assert_eq!(control.take(), Some("cursor_down"));
        assert_eq!(control.take(), Some("open_file"));
        assert_eq!(control.take(), None);
    }

    #[test]
    fn unrecognized_input() {
        let mut control = KeyboardControl::new(maplit::hashmap! {
            "of" => "open_file",
        });

        control.on_text("j");
        control.on_text("o");
        control.on_text("f");

        assert_eq!(control.take(), Some("open_file"));
    }

    #[test]
    fn unrecognized_partial_input() {
        let mut control = KeyboardControl::new(maplit::hashmap! {
            "of" => "open_file",
        });

        control.on_text("o");
        control.on_text("r");
        control.on_text("o");
        control.on_text("f");

        assert_eq!(control.take(), Some("open_file"));
    }

    #[test]
    fn prefixed_free() {
        let mut control = KeyboardControl::new(maplit::hashmap! {
            "f" => "cursor_down",
            "of" => "open_file",
        });

        control.on_text("o");
        control.on_text("f");

        assert_eq!(control.take(), Some("open_file"));
    }

    #[test]
    fn three_char_sequence() {
        let mut control = KeyboardControl::new(maplit::hashmap! {
            "j" => "cursor_down",
            "oof" => "open_file",
        });

        control.on_text("o");
        control.on_text("o");
        control.on_text("j");

        assert_eq!(control.take(), Some("cursor_down"));
    }

    #[test]
    fn captial_char() {
        let mut control = KeyboardControl::new(maplit::hashmap! {
            "J" => "cursor_down",
        });

        control.on_text("J");

        assert_eq!(control.take(), Some("cursor_down"));
    }

    #[test]
    fn multi_letter_button() {
        let mut control = KeyboardControl::new(maplit::hashmap! {
            "<Tab>" => "cursor_down",
        });

        control.on_key(Key::Tab, Modifiers::default());

        assert_eq!(control.take(), Some("cursor_down"));
    }
}
