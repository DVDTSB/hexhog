use std::{fs::File, io::Write};

use crate::app::App;

impl App {
    pub fn quit(&mut self) {
        self.running = false;
    }
    //starting_line
    pub fn set_startingline(&mut self) {
        if self.cursor_y < self.starting_line + 5 {
            self.starting_line = self.cursor_y.saturating_sub(5);
        }
        if self.cursor_y > self.starting_line + self.frame_height.saturating_sub(1 + 5) {
            self.starting_line = self
                .cursor_y
                .saturating_sub(self.frame_height.saturating_sub(1 + 5));
        }
    }

    //cursor
    pub fn get_idx(&self) -> usize {
        self.cursor_y * 16 + self.cursor_x
    }

    pub fn set_idx(&mut self, idx: usize) {
        self.cursor_y = idx / 16;
        self.cursor_x = idx % 16;
    }

    pub fn move_up(&mut self) {
        self.cursor_y = self.cursor_y.saturating_sub(1);
    }
    pub fn move_down(&mut self) {
        self.cursor_y += 1;
        if self.cursor_y * 16 > self.data.len() {
            self.cursor_y -= 1;
        }
    }
    pub fn move_page_up(&mut self) {
        self.cursor_y = self.cursor_y.saturating_sub(self.frame_height);
    }
    pub fn move_page_down(&mut self) {
        self.cursor_y += self.frame_height;
        if self.cursor_y * 16 > self.data.len() {
            self.cursor_y -= self.frame_height;
        }
    }
    pub fn move_right(&mut self) {
        self.cursor_x += 1;
        if self.get_idx() > self.data.len() {
            self.cursor_x -= 1;
        }
        if self.cursor_x >= 16 {
            self.cursor_x = 0;
            self.cursor_y += 1;
        }
    }
    pub fn move_left(&mut self) {
        if self.cursor_x == 0 {
            if self.cursor_y == 0 {
                return;
            }
            self.cursor_x = 15;
            self.cursor_y = self.cursor_y.saturating_sub(1);
        } else {
            self.cursor_x -= 1;
        }
    }

    //selection
    pub fn selection_range(&self) -> (usize, usize) {
        if !self.is_selecting {
            return (self.get_idx(), self.get_idx());
        }
        (
            self.get_idx().min(self.selection_start),
            self.get_idx()
                .max(self.selection_start)
                .min(self.data.len() - 1),
        )
    }

    pub fn get_selection_data(&self) -> Vec<u8> {
        let (x, y) = self.selection_range();
        self.data[x..(y + 1)].to_vec()
    }

    //buffer
    pub fn insert_to_buffer(&mut self, c: char) {
        let c = c.to_ascii_uppercase();
        if self.buffer[0] == ' ' {
            self.buffer[0] = c;
        } else if self.buffer[1] == ' ' {
            self.buffer[1] = c;
        }
    }

    pub fn buffer_to_u8(&self) -> u8 {
        let mut s = String::new();
        s.push(self.buffer[0]);
        s.push(self.buffer[1]);
        u8::from_str_radix(&s, 16).unwrap()
    }

    //data functions
    pub fn replace_data(&mut self, idx: usize, new: Vec<u8>) {
        for (i, b) in new.iter().enumerate() {
            let pos = idx + i;
            if pos < self.data.len() {
                self.data[pos] = *b;
            } else {
                self.data.push(*b);
            }
        }
    }

    pub fn insert_data(&mut self, idx: usize, new: Vec<u8>) {
        for (i, b) in new.iter().enumerate() {
            self.data.insert(idx + i, *b);
        }
    }

    pub fn delete_data(&mut self, idx: usize, amt: usize) {
        for _ in 0..amt {
            if idx < self.data.len() {
                self.data.remove(idx);
            } else {
                break;
            }
        }
    }

    pub fn save(&mut self) {
        File::create(self.file_name.clone())
            .unwrap()
            .write_all(&self.data)
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::state::AppState;
    use crate::config::Config;

    fn make_app(data: Vec<u8>) -> App {
        App {
            config: Config::default(),
            file_name: String::new(),
            data,
            starting_line: 0,
            cursor_x: 0,
            cursor_y: 0,
            frame_height: 20,
            running: true,
            state: AppState::Move,
            buffer: [' ', ' '],
            changes: Vec::new(),
            made_changes: Vec::new(),
            is_inserting: false,
            is_selecting: false,
            selection_start: 0,
            clipboard: Vec::new(),
        }
    }

    #[test]
    fn get_idx_origin() {
        let app = make_app(vec![0u8; 32]);
        assert_eq!(app.get_idx(), 0);
    }

    #[test]
    fn get_idx_x_offset() {
        let mut app = make_app(vec![0u8; 32]);
        app.cursor_x = 5;
        assert_eq!(app.get_idx(), 5);
    }

    #[test]
    fn get_idx_y_offset() {
        let mut app = make_app(vec![0u8; 32]);
        app.cursor_y = 1;
        assert_eq!(app.get_idx(), 16);
    }

    #[test]
    fn set_idx_row_col() {
        let mut app = make_app(vec![0u8; 64]);
        app.set_idx(35);
        assert_eq!(app.cursor_y, 2);
        assert_eq!(app.cursor_x, 3);
    }

    #[test]
    fn move_right_increments_x() {
        let mut app = make_app(vec![0u8; 32]);
        app.move_right();
        assert_eq!(app.cursor_x, 1);
    }

    #[test]
    fn move_right_wraps_to_next_row() {
        let mut app = make_app(vec![0u8; 32]);
        app.cursor_x = 15;
        app.move_right();
        assert_eq!(app.cursor_x, 0);
        assert_eq!(app.cursor_y, 1);
    }

    #[test]
    fn move_left_decrements_x() {
        let mut app = make_app(vec![0u8; 32]);
        app.cursor_x = 5;
        app.move_left();
        assert_eq!(app.cursor_x, 4);
    }

    #[test]
    fn move_left_wraps_to_prev_row() {
        let mut app = make_app(vec![0u8; 32]);
        app.cursor_y = 1;
        app.move_left();
        assert_eq!(app.cursor_x, 15);
        assert_eq!(app.cursor_y, 0);
    }

    #[test]
    fn move_left_at_origin_does_nothing() {
        let mut app = make_app(vec![0u8; 32]);
        app.move_left();
        assert_eq!(app.cursor_x, 0);
        assert_eq!(app.cursor_y, 0);
    }

    #[test]
    fn move_down_increments_y() {
        let mut app = make_app(vec![0u8; 32]);
        app.move_down();
        assert_eq!(app.cursor_y, 1);
    }

    #[test]
    fn move_up_decrements_y() {
        let mut app = make_app(vec![0u8; 32]);
        app.cursor_y = 2;
        app.move_up();
        assert_eq!(app.cursor_y, 1);
    }

    #[test]
    fn move_up_at_zero_does_nothing() {
        let mut app = make_app(vec![0u8; 32]);
        app.move_up();
        assert_eq!(app.cursor_y, 0);
    }

    #[test]
    fn insert_to_buffer_first_char() {
        let mut app = make_app(vec![]);
        app.insert_to_buffer('a');
        assert_eq!(app.buffer[0], 'A');
        assert_eq!(app.buffer[1], ' ');
    }

    #[test]
    fn insert_to_buffer_second_char() {
        let mut app = make_app(vec![]);
        app.insert_to_buffer('f');
        app.insert_to_buffer('3');
        assert_eq!(app.buffer[0], 'F');
        assert_eq!(app.buffer[1], '3');
    }

    #[test]
    fn buffer_to_u8_max() {
        let mut app = make_app(vec![]);
        app.buffer = ['F', 'F'];
        assert_eq!(app.buffer_to_u8(), 255);
    }

    #[test]
    fn buffer_to_u8_zero() {
        let mut app = make_app(vec![]);
        app.buffer = ['0', '0'];
        assert_eq!(app.buffer_to_u8(), 0);
    }

    #[test]
    fn replace_data_modifies_byte() {
        let mut app = make_app(vec![0x00, 0x01, 0x02]);
        app.replace_data(1, vec![0xAB]);
        assert_eq!(app.data[1], 0xAB);
    }

    #[test]
    fn insert_data_inserts_byte() {
        let mut app = make_app(vec![0x00, 0x02]);
        app.insert_data(1, vec![0x01]);
        assert_eq!(app.data, vec![0x00, 0x01, 0x02]);
    }

    #[test]
    fn delete_data_removes_byte() {
        let mut app = make_app(vec![0x00, 0x01, 0x02]);
        app.delete_data(1, 1);
        assert_eq!(app.data, vec![0x00, 0x02]);
    }

    #[test]
    fn selection_range_when_not_selecting() {
        let mut app = make_app(vec![0u8; 32]);
        app.cursor_x = 3;
        app.cursor_y = 1;
        let (min, max) = app.selection_range();
        assert_eq!(min, 19);
        assert_eq!(max, 19);
    }

    #[test]
    fn selection_range_forward() {
        let mut app = make_app(vec![0u8; 32]);
        app.is_selecting = true;
        app.selection_start = 5;
        app.cursor_x = 10;
        app.cursor_y = 0;
        let (min, max) = app.selection_range();
        assert_eq!(min, 5);
        assert_eq!(max, 10);
    }

    #[test]
    fn selection_range_reverse() {
        let mut app = make_app(vec![0u8; 32]);
        app.is_selecting = true;
        app.selection_start = 10;
        app.cursor_x = 5;
        app.cursor_y = 0;
        let (min, max) = app.selection_range();
        assert_eq!(min, 5);
        assert_eq!(max, 10);
    }
}
