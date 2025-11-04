mod byte;
mod config;

use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
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
    starting_line: usize,
    cursor_x: usize,
    cursor_y: usize,
    frame_height: usize,
    running: bool,
    state: AppState,
    buffer: [char; 2],         //used for editing a byte
    changes: Vec<Change>,      //undos
    made_changes: Vec<Change>, //redos
    is_inserting: bool,
    is_selecting: bool,
    selection_start: usize,
    clipboard: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppState {
    Move,
    Edit,
    Help,
}

#[derive(Debug, Clone)]
pub enum Change {
    Edit(usize, Vec<u8>, Vec<u8>),
    Insert(usize, Vec<u8>),
    Delete(usize, Vec<u8>),
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
        let area = frame.area();
        let buffer = frame.buffer_mut();

        buffer.set_style(
            area,
            Style::default().bg(self.config.colorscheme.background),
        );

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

        self.frame_height = layout[1].height as usize;

        let mut addr_text = Text::default();
        let mut hex_text = Text::default();
        let mut ascii_text = Text::default();

        let mut offset = 0;

        for i in self.starting_line..self.starting_line + layout[1].height as usize {
            let row_start = i * 16;

            if row_start > self.data.len() {
                break;
            }

            let addr_style = if i == self.cursor_y {
                Style::default().fg(self.config.colorscheme.primary)
            } else {
                Style::default().fg(self.config.colorscheme.primary).dim()
            };

            addr_text
                .lines
                .push(Line::from(format!("{row_start:08X}").set_style(addr_style)));

            let mut hex_line = Vec::new();
            let mut ascii_line = Vec::new();

            for j in 0..16 {
                let pos = row_start + j - offset;
                if pos > self.data.len() {
                    break;
                }

                let cursor_here = i == self.cursor_y && j == self.cursor_x;
                let editing = matches!(self.state, AppState::Edit) && cursor_here;

                let span = if editing && offset == 0 {
                    offset = self.is_inserting as usize;

                    ascii_line
                        .push(" ".bg(self.config.colorscheme.primary));

                    Span::from(format!("{}{}", self.buffer[0], self.buffer[1]))
                        .fg(self.config.colorscheme.primary)
                        .reversed()
                } else if pos < self.data.len() {
                    let byte = Byte::new(self.data[pos as usize]);
                    let mut style = byte.get_style(&self.config);
                    style = if cursor_here {
                        style.reversed()
                    } else {
                        match self.is_selecting {
                            false => style,
                            true => {
                                let (x, y) = self.selection_range();
                                if x <= pos && pos <= y {
                                    style
                                        .bg(self.config.colorscheme.select)
                                        .fg(self.config.colorscheme.primary)
                                } else {
                                    style
                                }
                            }
                        }
                    };
                    ascii_line
                        .push(Span::from(byte.get_char(&self.config).to_string()).set_style(style));
                    byte.get_hex().set_style(style)
                } else if cursor_here {
                    Span::from("  ")
                        .fg(self.config.colorscheme.primary)
                        .reversed()
                } else {
                    continue;
                };

                hex_line.push(span);

                // spacing

                let spacing = if j == 7 {
                    "  "
                } else if j < 15 {
                    " "
                } else {
                    ""
                };

                hex_line.push(match self.is_selecting {
                    true => {
                        let (x, y) = self.selection_range();
                        if x <= pos && pos < y {
                            spacing.bg(self.config.colorscheme.select).into()
                        } else {
                            spacing.into()
                        }
                    }
                    _ => spacing.into(),
                })
            }

            hex_text.lines.push(Line::from(hex_line));
            ascii_text.lines.push(Line::from(ascii_line));
        }

        frame.render_widget(Paragraph::new(addr_text), columns[0]);
        frame.render_widget(
            Paragraph::new(hex_text).block(
                Block::new()
                    .borders(Borders::LEFT | Borders::RIGHT)
                    .border_style(Style::default().fg(self.config.colorscheme.border))
                    .padding(Padding::horizontal(1)),
            ),
            columns[1],
        );
        frame.render_widget(Paragraph::new(ascii_text), columns[2]);

        // render help popup
        if self.state == AppState::Help {
            let popup = Paragraph::new(
                "h - help       u - undo
q - quit       U - redo
i - insert     s - save
backspace - delete byte
pgup,pgdn - move screen
",
            )
            .fg(self.config.colorscheme.primary)
            .block(
                Block::bordered()
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .fg(self.config.colorscheme.primary)
                    .padding(Padding::uniform(1)),
            )
            .centered();

            let popup_layout = Layout::default()
                .direction(Direction::Horizontal)
                .flex(Flex::Center)
                .constraints(vec![Constraint::Length(31)])
                .split(frame.area());

            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .flex(Flex::Center)
                .constraints(vec![Constraint::Length(9)])
                .split(popup_layout[0]);

            frame.render_widget(Clear, popup_layout[0]);

            let buffer = frame.buffer_mut();
            buffer.set_style(
                popup_layout[0],
                Style::default().bg(self.config.colorscheme.background),
            );
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
                (_, KeyCode::PageUp) => self.move_page_up(),
                (_, KeyCode::PageDown) => self.move_page_down(),

                (_, KeyCode::Char('v')) => {
                    if self.is_selecting {
                        self.is_selecting = false;
                    } else {
                        self.is_selecting = true;
                        self.selection_start = self.get_idx();
                    }
                }

                (_, KeyCode::Esc) => {
                    self.is_selecting = false;
                }

                (_, KeyCode::Char('y')) => {
                    self.clipboard = self.get_selection_data();
                    self.is_selecting = false;
                }
                (_, KeyCode::Char('p')) => {
                    self.do_change(Change::Insert(self.get_idx(), self.clipboard.clone()));
                    self.selection_start = self.get_idx();
                    self.is_selecting = true;
                    self.set_idx(self.selection_start+self.clipboard.len()-1);
                }

                (_, KeyCode::Backspace) => {
                    let idx = self.get_idx();
                    let (x, y) = self.selection_range();
                    let old = self.data[x..(y + 1)].to_vec();

                    self.do_change(Change::Delete(idx, old));

                    //if where the cursor was now theres nothing then move it!
                    let new_idx = idx.min(self.data.len() - 1);
                    self.set_idx(new_idx);
                }
                (KeyModifiers::NONE, KeyCode::Char(c)) if c.is_ascii_hexdigit() => {
                    self.is_selecting = false;
                    self.state = AppState::Edit;
                    self.is_inserting = false;
                    self.insert_to_buffer(c);
                }

                /*
                //i really wanna do this but for some reason it doesnt work with numbers
                // for later!
                (KeyModifiers::SHIFT, KeyCode::Char(c)) if c.is_ascii_hexdigit() => {
                    self.state = AppState::Edit;
                    self.is_inserting = true;
                    self.insert_to_buffer(c);
                },
                */
                (_, KeyCode::Char('i')) => {
                    self.is_selecting = false;
                    self.state = AppState::Edit;
                    self.is_inserting = true;
                }

                (KeyModifiers::NONE, KeyCode::Char('u'))
                | (KeyModifiers::NONE, KeyCode::Char('U')) => {
                    self.is_selecting = false;
                    self.undo();
                }
                (KeyModifiers::SHIFT, KeyCode::Char('u'))
                | (KeyModifiers::SHIFT, KeyCode::Char('U')) => {
                    self.is_selecting = false;
                    self.redo();
                }
                (_, KeyCode::Char('s')) | (_, KeyCode::Char('S')) => self.save(),
                (_, KeyCode::Char('h')) | (_, KeyCode::Char('H')) => {
                    self.is_selecting = false;
                    self.state = AppState::Help;
                }

                _ => {}
            },
            AppState::Edit => match (key.modifiers, key.code) {
                (_, KeyCode::Esc) | (_, KeyCode::Backspace) => {
                    self.state = AppState::Move;
                    self.buffer = [' ', ' '];
                }
                (_, KeyCode::Char(c)) if c.is_ascii_hexdigit() => {
                    self.insert_to_buffer(c);
                    if self.buffer[0] != ' ' && self.buffer[1] != ' ' {
                        self.state = AppState::Move;
                        let idx = self.get_idx();
                        let new = self.buffer_to_u8();

                        if self.is_inserting {
                            self.do_change(Change::Insert(idx, vec![new]));
                        } else {
                            if idx >= self.data.len() {
                                self.data.push(new);
                            } else {
                                let old = self.data[idx];
                                self.do_change(Change::Edit(idx, vec![old], vec![new]))
                            }
                        }
                        self.buffer = [' ', ' '];
                        self.move_right();
                        self.is_inserting = false;
                    }
                }
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

    fn get_idx(&self) -> usize {
        self.cursor_y * 16 + self.cursor_x
    }

    fn set_idx(&mut self, idx:usize) {
        self.cursor_y = idx/16;
        self.cursor_x = idx%16;
    }

    fn move_up(&mut self) {
        self.cursor_y = self.cursor_y.saturating_sub(1);
    }
    fn move_down(&mut self) {
        self.cursor_y += 1;
        if self.cursor_y * 16 > self.data.len() {
            self.cursor_y -= 1;
        }
    }
    fn move_page_up(&mut self) {
        self.cursor_y = self.cursor_y.saturating_sub(self.frame_height);
    }
    fn move_page_down(&mut self) {
        self.cursor_y += self.frame_height;
        if self.cursor_y * 16 > self.data.len() {
            self.cursor_y -= self.frame_height;
        }
    }
    fn move_right(&mut self) {
        self.cursor_x += 1;
        if self.get_idx() >= self.data.len() + 1 {
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

    fn selection_range(&self) -> (usize, usize) {
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

    fn get_selection_data(&self) -> Vec<u8> {
        let (x,y) = self.selection_range();
        self.data[x..(y+1)].to_vec()
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

    fn replace_data(&mut self, idx: usize, new: Vec<u8>) {
        for (i, b) in new.iter().enumerate() {
            let pos = idx + i;
            if pos < self.data.len() {
                self.data[pos] = *b;
            } else {
                self.data.push(*b);
            }
        }
    }

    fn insert_data(&mut self, idx: usize, new: Vec<u8>) {
        for (i, b) in new.iter().enumerate() {
            self.data.insert(idx + i, *b);
        }
    }

    fn delete_data(&mut self, idx: usize, amt: usize) {
        for _ in 0..amt {
            if idx < self.data.len() {
                self.data.remove(idx);
            } else {
                break;
            }
        }
    }

    fn do_change(&mut self, change: Change) {
        self.changes.push(change.clone());
        match change {
            Change::Edit(idx, _old, new) => self.replace_data(idx, new),
            Change::Insert(idx, new) => self.insert_data(idx, new),
            Change::Delete(idx, old) => self.delete_data(idx, old.len()),
        }
    }

    fn undo_change(&mut self, change: Change) {
        self.made_changes.push(change.clone());
        match change {
            Change::Edit(idx, old, _new) => self.replace_data(idx, old),
            Change::Insert(idx, new) => self.delete_data(idx, new.len()),
            Change::Delete(idx, old) => self.insert_data(idx, old),
        }
    }

    fn undo(&mut self) {
        if let Some(change) = self.changes.pop() {
            self.undo_change(change);
        }
    }

    fn redo(&mut self) {
        if let Some(change) = self.made_changes.pop() {
            self.do_change(change);
        }
    }

    fn save(&mut self) {
        File::create(self.file_name.clone())
            .unwrap()
            .write_all(&self.data)
            .unwrap();
    }
}
