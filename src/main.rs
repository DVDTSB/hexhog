mod byte;
mod config;

use std::{
    cmp::min,
    fs::File,
    io::{Read, Write},
};

use byte::Byte;
use clap::Parser;
use color_eyre::Result;
use config::Config;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Flex, Layout},
    prelude::Alignment,
    style::{Style, Styled, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Padding, Paragraph},
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let config_file_path = dirs::config_dir()
        .unwrap()
        .join("hexhog")
        .join("config.toml");

    let config = Config::read_config(config_file_path.to_str().unwrap()).unwrap_or_else(|e| {
        eprintln!("Error reading config: {e}");
        eprintln!("Using default config");
        Config::default()
    });

    let app = App::new(args, config)?;
    let terminal = ratatui::init();
    let result = app.run(terminal);
    ratatui::restore();
    result
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    file: String,
}

pub struct App {
    config: Config,
    file_name: String,
    data: Vec<u8>,
    starting_line: u32,
    cursor_x: u32,
    cursor_y: u32,
    frame_height: u32,
    running: bool,
    state: AppState,
    buffer: [char; 2],         //used for editing a byte
    changes: Vec<Change>,      //undos
    made_changes: Vec<Change>, //redos
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppState {
    Move,
    Edit,
    Insert,
    Help,
}

#[derive(Debug)]
pub enum Change {
    Edit(usize, u8, u8),
    Insert(usize, u8),
    Delete(usize, u8),
}

impl App {
    pub fn new(args: Args, config: Config) -> Result<Self> {
        let mut file = File::open(&args.file)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

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
            config: config,
        })
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;

            if self.cursor_y < self.starting_line + 5 {
                self.starting_line = self.cursor_y.saturating_sub(5);
            }
            if self.cursor_y > self.starting_line + self.frame_height.saturating_sub(1 + 5) {
                self.starting_line = self
                    .cursor_y
                    .saturating_sub(self.frame_height.saturating_sub(1 + 5));
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(5),
                Constraint::Length(1),
            ])
            .split(frame.area());

        let title = Paragraph::new(format!(" hexhog ─ {} ", self.file_name))
            .alignment(Alignment::Center)
            .fg(self.config.colorscheme.accent);
        frame.render_widget(title, layout[0]);

        let status_text = format!(
            " h - help | state: {:?} │ cursor: {:08X} │ size: {} bytes ",
            self.state,
            self.cursor_x + self.cursor_y * 16,
            self.data.len(),
        );
        let status = Paragraph::new(status_text)
            .alignment(Alignment::Center)
            .fg(self.config.colorscheme.accent);
        frame.render_widget(status, layout[2]);

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(8),
                Constraint::Length(48 + 2 + 2),
                Constraint::Length(16),
            ])
            .flex(Flex::Center)
            .split(layout[1]);

        self.frame_height = layout[1].height as u32;

        let mut addr_text = Text::default();
        let mut hex_text = Text::default();
        let mut ascii_text = Text::default();

        let mut offset = 0;

        for i in self.starting_line..(self.starting_line + layout[1].height as u32) {
            if i * 16 >= self.data.len() as u32 {
                break;
            }

            let addr_style = if i == self.cursor_y {
                Style::default().gray().dim()
            } else {
                Style::default().dark_gray()
            };

            addr_text
                .lines
                .push(Line::from(format!("{:08X}", i * 16).set_style(addr_style)));

            let mut hex_line = Vec::new();
            let mut ascii_line = Vec::new();

            for j in (i * 16)..min(i * 16 + 16, self.data.len() as u32) {
                let j_pos = j;
                let j_idx = j - offset;

                let byte = Byte::new(self.data[j_idx as usize]);

                let mut style = byte.get_style(&self.config);

                // reverse if cursor is here
                if i == self.cursor_y && j_pos % 16 == self.cursor_x {
                    style = style.reversed();
                } else {
                    style = style.not_reversed();
                }

                // if editing
                if i == self.cursor_y
                    && j % 16 == self.cursor_x
                    && (self.state == AppState::Edit || self.state == AppState::Insert)
                    && offset == 0
                {
                    hex_line.push(
                        Span::from(format!("{}{}", self.buffer[0], self.buffer[1]))
                            .black()
                            .on_white(),
                    );
                    offset = (self.state == AppState::Insert) as u32;
                } else {
                    hex_line.push(byte.get_hex().set_style(style));
                }

                // space between columns
                if j_pos % 16 == 7 {
                    hex_line.push("  ".into());
                } else if j_pos % 16 < 15 {
                    hex_line.push(" ".into());
                }

                let ch = byte.get_char(&self.config);
                ascii_line.push(Span::from(ch.to_string()).set_style(style));
            }

            hex_text.lines.push(Line::from(hex_line));
            ascii_text.lines.push(Line::from(ascii_line));
        }

        frame.render_widget(Paragraph::new(addr_text), columns[0]);
        frame.render_widget(
            Paragraph::new(hex_text).block(
                Block::new()
                    .borders(Borders::LEFT | Borders::RIGHT)
                    .padding(Padding::horizontal(1)),
            ),
            columns[1],
        );
        frame.render_widget(Paragraph::new(ascii_text), columns[2]);

        // render help popup
        if self.state == AppState::Help {
            let popup =
                Paragraph::new("h - help\nq - quit\ni - insert\nu - undo\nU - redo\ns - save")
                    .gray()
                    .block(
                        Block::bordered()
                            .border_type(ratatui::widgets::BorderType::Rounded)
                            .padding(Padding::uniform(1)),
                    )
                    .centered();

            let popup_layout = Layout::default()
                .direction(Direction::Horizontal)
                .flex(Flex::Center)
                .constraints(vec![Constraint::Length(16)])
                .split(frame.area());

            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .flex(Flex::Center)
                .constraints(vec![Constraint::Length(10)])
                .split(popup_layout[0]);

            frame.render_widget(Clear, popup_layout[0]);
            frame.render_widget(popup, popup_layout[0]);
        }
    }

    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            _ => {}
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match self.state {
            AppState::Move => match (key.modifiers, key.code) {
                (_, KeyCode::Char('q')) => self.quit(),
                (_, KeyCode::Right) => self.move_right(),
                (_, KeyCode::Left) => self.move_left(),
                (_, KeyCode::Up) => self.move_up(),
                (_, KeyCode::Down) => self.move_down(),
                (KeyModifiers::NONE, KeyCode::Char(c)) if c.is_ascii_hexdigit() => {
                    self.state = AppState::Edit;
                    self.insert_to_buffer(c);
                }
                (_, KeyCode::Char('i')) => {
                    self.state = AppState::Insert;
                }

                (KeyModifiers::NONE, KeyCode::Char('u'))
                | (KeyModifiers::NONE, KeyCode::Char('U')) => self.undo(),
                (KeyModifiers::SHIFT, KeyCode::Char('u'))
                | (KeyModifiers::SHIFT, KeyCode::Char('U')) => self.redo(),
                (_, KeyCode::Char('s')) | (_, KeyCode::Char('S')) => self.save(),
                (_, KeyCode::Char('h')) | (_, KeyCode::Char('H')) => {
                    self.state = AppState::Help;
                }

                _ => {}
            },
            AppState::Edit => match (key.modifiers, key.code) {
                (_, KeyCode::Esc) => {
                    self.state = AppState::Move;
                    self.buffer = [' ', ' '];
                }
                (_, KeyCode::Char(c)) if c.is_ascii_hexdigit() => {
                    self.insert_to_buffer(c);
                    if self.buffer[0] != ' ' && self.buffer[1] != ' ' {
                        self.state = AppState::Move;
                        let idx = (self.cursor_y * 16 + self.cursor_x) as usize;
                        let old = self.data[idx];
                        let new = self.buffer_to_u8();
                        self.data[idx] = new;
                        self.buffer = [' ', ' '];
                        self.changes.push(Change::Edit(idx, old, new))
                    }
                }
                (_, KeyCode::Backspace) => {}
                _ => {}
            },
            AppState::Insert => match (key.modifiers, key.code) {
                (_, KeyCode::Esc) => {
                    self.state = AppState::Move;
                    self.buffer = [' ', ' '];
                }
                (_, KeyCode::Char(c)) if c.is_ascii_hexdigit() => {
                    self.insert_to_buffer(c);

                    if self.buffer[0] != ' ' && self.buffer[1] != ' ' {
                        self.state = AppState::Move;
                        let idx = (self.cursor_y * 16 + self.cursor_x) as usize;
                        let new = self.buffer_to_u8();
                        self.data.insert(idx, new);
                        self.buffer = [' ', ' '];
                        self.changes.push(Change::Insert(idx, new))
                    }
                }
                (_, KeyCode::Backspace) => {}
                _ => {}
            },
            AppState::Help => {
                self.state = AppState::Move;
            }
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }

    fn move_up(&mut self) {
        self.cursor_y = self.cursor_y.saturating_sub(1);
    }
    fn move_down(&mut self) {
        self.cursor_y += 1;
        if self.cursor_y * 16 > self.data.len() as u32 {
            self.cursor_y -= 1;
        }
    }
    fn move_right(&mut self) {
        self.cursor_x += 1;
        if self.cursor_y * 16 + self.cursor_x >= self.data.len() as u32 {
            self.cursor_x -= 1;
        }
        if self.cursor_x >= 16 {
            self.cursor_x = 0;
            self.cursor_y += 1;
        }
    }
    fn move_left(&mut self) {
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

    fn insert_to_buffer(&mut self, c: char) {
        let c = c.to_ascii_uppercase();
        if self.buffer[0] == ' ' {
            self.buffer[0] = c;
        } else if self.buffer[1] == ' ' {
            self.buffer[1] = c;
        }
    }

    fn buffer_to_u8(&self) -> u8 {
        let mut s = String::new();
        s.push(self.buffer[0]);
        s.push(self.buffer[1]);
        u8::from_str_radix(&s, 16).unwrap()
    }

    fn undo(&mut self) {
        let change = self.changes.pop();
        if let Some(c) = change {
            match c {
                Change::Edit(idx, old, _) => {
                    self.data[idx] = old;
                }
                Change::Insert(idx, _) => {
                    self.data.remove(idx);
                }
                Change::Delete(idx, a) => {
                    self.data.insert(idx, a);
                }
            }
        }
    }

    fn redo(&mut self) {
        let change = self.made_changes.pop();
        if let Some(c) = change {
            match c {
                Change::Edit(idx, _, new) => {
                    self.data[idx] = new;
                }
                Change::Insert(idx, u8) => {
                    self.data.insert(idx, u8);
                }
                Change::Delete(idx, _) => {
                    self.data.remove(idx);
                }
            }
        }
    }

    fn save(&mut self) {
        File::create(self.file_name.clone())
            .unwrap()
            .write_all(&self.data)
            .unwrap();
    }
}
