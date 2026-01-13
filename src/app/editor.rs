use crate::{
    app::{Window, data::EditorData, history::History},
    config::Config,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    self,
    layout::{Constraint, Direction, Layout},
    style::{Styled, Stylize},
    text::{Line, Text},
};

pub enum InputMode {
    Hex,
    Ascii
}
pub struct Editor {
    pub config: Config,

    pub cursor_position: usize,

    pub is_editing: bool,
    pub is_inserting: bool,
    pub is_small: bool, // if i use this inside another window
    pub input_mode: InputMode,
    pub is_focused: bool,
    pub is_open: bool,

    pub group_size: usize,
    pub group_count_wanted: usize,
    pub group_count: usize,

    pub editor_width: usize,
    pub editor_height: usize,

    pub starting_line: usize,
    pub lines_on_screen: usize,

    pub show_text: bool,
    pub show_hex: bool,
    pub show_address: bool,

    pub address_bits: usize,

    

    pub editor_data: EditorData,
    pub history: History<EditorData>,

    buffer: Vec<u8>,
}

impl Editor {
    fn get_cursor_position(&self) -> (usize, usize) {
        let cursor_row = (self.cursor_position / (self.group_size * self.group_count));
        let cursor_col = (self.cursor_position % (self.group_size * self.group_count));
        (cursor_row, cursor_col)
    }
}

impl Window for Editor {
    fn is_open(&self) -> bool {
        self.is_open
    }

    fn set_open(&mut self, open: bool) {
        self.is_open = open;
    }

    fn set_focus(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let group_count = (frame.area().width as usize
            - if self.show_hex { 1 } else { 0 }
            - if self.show_text { 1 } else { 0 }
            - if self.show_address {
                self.address_bits + 1
            } else {
                0
            }
            + 1)
            / ((self.group_size * 3 + 1) * (self.show_hex as usize)
                + self.group_size * (self.show_text as usize));
        self.group_count = group_count.min(self.group_count_wanted);

        let mut addr_lines = Text::default();
        let mut hex_lines = Text::default();
        let mut text_lines = Text::default();

        let row_data_size = self.group_size * self.group_count;

        let (x, y) = self.get_cursor_position();

        let mut offset = 0;

        for i in self.starting_line..self.starting_line + self.lines_on_screen {
            let row_start = i * row_data_size;

            if row_start > self.buffer.len() {
                break;
            }

            if self.show_address {
                let addr_style = if i == y {
                    ratatui::style::Style::default().fg(self.config.colorscheme.primary)
                } else {
                    ratatui::style::Style::default()
                        .fg(self.config.colorscheme.primary)
                        .dim()
                };
                let address = format!("{:0width$x}", row_start, width = self.address_bits);
                addr_lines
                    .lines
                    .push(Line::from(address + " ").set_style(addr_style));
            }

            for j in 0..row_data_size {
                let pos = row_start+j-offset;
                if pos > self.buffer.len() {
                    break;
                }

                let cursor_here = i==y && j==x;

                let is_editing = cursor_here && self.is_editing;




            }
        }

        let need_width_hex = (3 * self.group_size + 1) * self.group_count;
        let need_width_text = self.group_size * self.group_count;
        let need_width_address = self.address_bits;
    }

    fn handle_event(&mut self, event: Event) -> color_eyre::Result<()> {
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('i') => {
                    self.is_inserting = !self.is_inserting;
                }
                KeyCode::Char(c) if c.is_ascii_hexdigit() => {}
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }
}
