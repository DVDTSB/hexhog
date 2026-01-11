use crate::app::Window;
use crossterm::event::{Event, KeyEvent, KeyCode, KeyModifiers};
use ratatui;


pub struct Editor {
    pub cursor_position: usize,

    pub is_inserting: bool,
    pub is_small: bool, // if i use this inside another window
    pub is_focused: bool,
    pub is_open: bool,

    pub group_size: usize,
    pub group_count: usize,


    pub editor_width: usize,
    pub editor_height: usize,

    pub starting_line: usize,
    pub lines_on_screen: usize,

    pub show_text: bool,
    pub show_address: bool,

    pub is_editing: bool,

    buffer: Vec<u8>,

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
    }

    fn handle_event(&mut self, event: Event) -> color_eyre::Result<()> {
        match event {
            Event::Key(key_event) => {
                match key_event.code {
                    KeyCode::Char('i') => {
                        self.is_inserting = !self.is_inserting;
                    },
                    KeyCode::Char(c) if c.is_ascii_hexdigit() => {
                        
                    },
                    _ => {}
                }
                
            }
            _ => {}
        }
        Ok(())
    }
}