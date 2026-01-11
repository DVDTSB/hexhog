use crate::app::window::Window;
use crate::config::Config;
use clap::Parser;
use color_eyre::Result;
use ratatui::DefaultTerminal;
use std::{fs::File, io::Read, path::Path};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    pub file: String,
}

pub struct App {
    pub config: Config,
    pub file_name: String,
    pub data: Vec<u8>,
    pub running: bool,
    pub windows: Vec<Box<dyn Window>>,
    pub focus_index: usize,
}

impl App {
    pub fn new(args: Args, config: Config) -> Result<Self> {
        let path = Path::new(&args.file);
        let mut data = Vec::new();

        if path.exists() {
            let mut file = File::open(&args.file)?;
            file.read_to_end(&mut data)?;
        }

        let mut windows: Vec<Box<dyn Window>> = Vec::new();

        windows.push(Box::new(crate::app::window::TextWindow { is_open: true, is_focused: true, title: "Welcome".to_string(), text: "Welcome to HexHog!\nHope you like your stay here!".to_string() }));

        Ok(Self {
            file_name: args.file,
            running: true,
            data,
            config,
            windows: windows,
            focus_index: 0,
        })
    }
    
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }
}
