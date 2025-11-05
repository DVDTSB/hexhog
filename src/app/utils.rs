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
        if self.get_idx() >= self.data.len() + 1 {
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
