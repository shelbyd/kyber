use crate::Direction;

#[derive(Default)]
pub struct Cursor {
    col: usize,
    row: usize,
}

// TODO(shelbyd): This allows scrolling way past the end of the file and lines.
impl Cursor {
    pub fn byte_pos(&self, text: &str) -> usize {
        byte_pos(text, self.row, self.col)
    }

    pub fn do_move(&mut self, direction: Direction) {
        match direction {
            Direction::Right => self.col += 1,
            Direction::Left => self.col = self.col.saturating_sub(1),

            Direction::Down => self.row += 1,
            Direction::Up => self.row = self.row.saturating_sub(1),
        }
    }
}

fn byte_pos(s: &str, row: usize, col: usize) -> usize {
    let (this_line, tail) = match s.split_once("\n") {
        Some((l, t)) => (l, Some(t)),
        None => (s, None),
    };

    if row > 0 {
        if let Some(tail) = tail {
            return this_line.len() + 1 + byte_pos(tail, row - 1, col);
        }
    }

    col.clamp(0, this_line.len().saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_at_0() {
        assert_eq!(Cursor::default().byte_pos(""), 0);
    }

    #[test]
    fn move_right_puts_at_one() {
        let mut cursor = Cursor::default();

        cursor.do_move(Direction::Right);

        assert_eq!(cursor.byte_pos("foo"), 1);
    }

    #[test]
    fn does_not_move_past_content() {
        let mut cursor = Cursor::default();

        cursor.do_move(Direction::Right);

        assert_eq!(cursor.byte_pos("x"), 0);
    }

    #[test]
    fn down_goes_to_next_line() {
        let mut cursor = Cursor::default();

        cursor.do_move(Direction::Down);

        assert_eq!(cursor.byte_pos("x\ny"), 2);
    }

    #[test]
    fn windows_newlines() {
        let mut cursor = Cursor::default();

        cursor.do_move(Direction::Down);

        assert_eq!(cursor.byte_pos("x\r\ny"), 3);
    }

    #[test]
    fn col_past_first_line() {
        let mut cursor = Cursor::default();

        cursor.do_move(Direction::Right);
        cursor.do_move(Direction::Right);
        cursor.do_move(Direction::Right);

        assert_eq!(cursor.byte_pos("x\ny"), 0);
    }

    #[test]
    fn row_past_last_line() {
        let mut cursor = Cursor::default();

        cursor.do_move(Direction::Down);

        assert_eq!(cursor.byte_pos("foo"), 0);
    }

    #[test]
    fn all_motions_cancel() {
        let mut cursor = Cursor::default();

        cursor.do_move(Direction::Down);
        cursor.do_move(Direction::Right);
        cursor.do_move(Direction::Up);
        cursor.do_move(Direction::Left);

        assert_eq!(cursor.byte_pos("12\n34"), 0);
    }
}
