use std::{fs::File, io::Read, path::Path};
use clap::Parser;
use color_eyre::Result;
use ratatui::DefaultTerminal;
use crate::config::Config;
use super::change::Change;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    pub file: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppState {
    Move,
    Edit,
    Help,
}

pub struct App {
    pub config: Config,
    pub file_name: String,
    pub data: Vec<u8>,
    pub starting_line: usize,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub frame_height: usize,
    pub running: bool,
    pub state: AppState,
    pub buffer: [char; 2],
    pub changes: Vec<Change>,
    pub made_changes: Vec<Change>,
    pub is_inserting: bool,
    pub is_selecting: bool,
    pub selection_start: usize,
    pub clipboard: Vec<u8>,
}

impl App {
    pub fn new(args: Args, config: Config) -> Result<Self> {
        let path = Path::new(&args.file);
        let mut data = Vec::new();

        if path.exists() {
            let mut file = File::open(&args.file)?;
            file.read_to_end(&mut data)?;
        }

        Ok(Self {
            file_name: args.file,
            running: true,
            data,
            starting_line: 0,
            cursor_x: 0,
            cursor_y: 0,
            frame_height: 0,
            state: AppState::Move,
            buffer: [' ', ' '],
            changes: Vec::new(),
            made_changes: Vec::new(),
            config,
            is_inserting: false,
            is_selecting: false,
            selection_start: 0,
            clipboard: Vec::new(),
        })
    }
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
            //maybe this will become an update() func if i need more stuff
            self.set_startingline();
        }
        Ok(())
    }
}
