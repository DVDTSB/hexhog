use std::{cmp::min, fs::File, io::Read};

use clap::Parser;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Flex, Layout},
    prelude::Alignment,
    style::{Style, Styled, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Padding, Paragraph},
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let terminal = ratatui::init();
    let result = App::new(args).run(terminal);
    ratatui::restore();
    result
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    file: String,
}

#[derive(Debug, Default)]
pub struct App {
    file_name: String,
    data: Vec<u8>,
    starting_line: u32,
    cursor_x: u32,
    cursor_y: u32,
    frame_height: u32,
    running: bool,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum HexType {
    Zero,
    NotPrintable,
    Printable,
    NotAscii,
}

impl App {
    pub fn new(args: Args) -> Self {
        let mut file = File::open(&args.file).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        Self {
            file_name: args.file,
            running: true,
            data,
            starting_line: 0,
            cursor_x: 0,
            cursor_y: 0,
            frame_height: 0,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;

            if self.cursor_y < self.starting_line {
                self.starting_line = self.cursor_y;
            }
            if self.cursor_y > self.starting_line + self.frame_height - 1 {
                self.starting_line = self.cursor_y - self.frame_height + 1;
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
            .bold()
            .blue();
        frame.render_widget(title, layout[0]);

        let status_text = format!(
            " q - quit │ line: {} │ cursor: {},{} │ size: {} bytes ",
            self.starting_line,
            self.cursor_x,
            self.cursor_y,
            self.data.len(),
        );
        let status = Paragraph::new(status_text)
            .alignment(Alignment::Center)
            .blue();
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

        for i in self.starting_line..(self.starting_line + layout[1].height as u32) {
            if i * 16 >= self.data.len() as u32 {
                break;
            }

            addr_text.lines.push(Line::from(format!("{:08X}", i * 16)));

            let mut hex_line = Vec::new();
            let mut ascii_line = Vec::new();

            for j in (i * 16)..min(i * 16 + 16, self.data.len() as u32) {
                let hextype = match self.data[j as usize] {
                    0x00 => HexType::Zero,
                    b if b.is_ascii_graphic() => HexType::Printable,
                    b if b.is_ascii() => HexType::NotPrintable,
                    _ => HexType::NotAscii,
                };

                let mut style = match hextype {
                    HexType::Zero => Style::default().dark_gray(),
                    HexType::NotPrintable => Style::default().blue(),
                    HexType::Printable => Style::default().cyan(),
                    HexType::NotAscii => Style::default().yellow(),
                };

                if i == self.cursor_y && j % 16 == self.cursor_x {
                    style = style.reversed();
                } else {
                    style = style.not_reversed();
                }

                hex_line.push(Span::from(
                    format!("{:02X}", self.data[j as usize]).set_style(style),
                ));
                if j % 16 == 7 {
                    hex_line.push("  ".into());
                } else if j % 16 < 15 {
                    hex_line.push(" ".into());
                }

                let ch = match hextype {
                    HexType::Zero => '0',
                    HexType::Printable => self.data[j as usize] as char,
                    _ => '.',
                };
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
    }

    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            _ => {}
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (KeyModifiers::NONE, KeyCode::Right) => {
                self.cursor_x += 1;
                if self.cursor_y * 16 + self.cursor_x >= self.data.len() as u32 {
                    self.cursor_x -= 1;
                }
                if self.cursor_x >= 16 {
                    self.cursor_x = 0;
                    self.cursor_y += 1;
                }
            }
            (KeyModifiers::NONE, KeyCode::Left) => {
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

            (KeyModifiers::NONE, KeyCode::Up) => {
                self.cursor_y = self.cursor_y.saturating_sub(1);
            }

            (KeyModifiers::NONE, KeyCode::Down) => {
                self.cursor_y += 1;
                if self.cursor_y * 16 > self.data.len() as u32 {
                    self.cursor_y -= 1;
                }
            }
            _ => {}
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }
}
