use crate::app::{App, state::AppState};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout},
    prelude::Alignment,
    style::{Style, Styled, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Padding, Paragraph},
};

use crate::byte::Byte;

impl App {
    pub fn render(&mut self, frame: &mut Frame) {
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

        let used_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(8+48+2+2+16),
            ])
            .flex(Flex::Center)
            .split(layout[2]);

        let status_text = format!(
            " h - help │ cursor: {:08X} │ size: {} bytes ",
            self.get_idx(),
            self.data.len(),
        );
        let status = Paragraph::new(status_text)
            .alignment(Alignment::Center)
            .fg(self.config.colorscheme.accent).reversed();

        frame.render_widget(status, used_area[0]);

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

                    ascii_line.push(" ".bg(self.config.colorscheme.primary));

                    Span::from(format!("{}{}", self.buffer[0], self.buffer[1]))
                        .fg(self.config.colorscheme.primary)
                        .reversed()
                } else if pos < self.data.len() {
                    let byte = Byte::new(self.data[pos as usize]);
                    let mut style = byte.get_style(&self.config);
                    style = if cursor_here {
                        match self.is_selecting {
                            false => style.reversed(),
                            true => style.fg(self.config.colorscheme.primary).reversed()
                        }
                        
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
            use ratatui::text::{Line, Span, Text};

            let accent = self.config.colorscheme.accent;
            let primary = self.config.colorscheme.primary;

            let lines = vec![
                Line::from(vec![
                    Span::styled("h", Style::default().fg(accent)),
                    Span::styled(" - help      ", Style::default().fg(primary)),
                    Span::styled("u", Style::default().fg(accent)),
                    Span::styled(" - undo     ", Style::default().fg(primary)),
                    Span::styled("v", Style::default().fg(accent)),
                    Span::styled(" - select", Style::default().fg(primary)),
                ]),
                Line::from(vec![
                    Span::styled("q", Style::default().fg(accent)),
                    Span::styled(" - quit      ", Style::default().fg(primary)),
                    Span::styled("U", Style::default().fg(accent)),
                    Span::styled(" - redo     ", Style::default().fg(primary)),
                    Span::styled("y", Style::default().fg(accent)),
                    Span::styled(" - copy", Style::default().fg(primary)),
                ]),
                Line::from(vec![
                    Span::styled("i", Style::default().fg(accent)),
                    Span::styled(" - insert    ", Style::default().fg(primary)),
                    Span::styled("s", Style::default().fg(accent)),
                    Span::styled(" - save     ", Style::default().fg(primary)),
                    Span::styled("p", Style::default().fg(accent)),
                    Span::styled(" - paste", Style::default().fg(primary)),
                ]),
                Line::from(vec![
                    Span::styled("bs", Style::default().fg(accent)),
                    Span::styled(" - delete   ", Style::default().fg(primary)),
                    Span::styled("pgup,pgdn", Style::default().fg(accent)),
                    Span::styled(" - move screen", Style::default().fg(primary)),
                ]),
            ];

            let popup = Paragraph::new(Text::from(lines)).block(
                Block::bordered()
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .fg(primary)
                    .padding(Padding::symmetric(4, 1))
                    .title_top(Line::from(vec![
                        Span::styled("──── ", Style::default().fg(primary)),
                        Span::styled("help ", Style::default().fg(accent)),
                    ])),
            );

            let popup_layout = Layout::default()
                .direction(Direction::Horizontal)
                .flex(Flex::End)
                .constraints(vec![Constraint::Length(47)])
                .split(layout[1]);

            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .flex(Flex::End)
                .constraints(vec![Constraint::Length(8)])
                .split(popup_layout[0]);

            frame.render_widget(Clear, popup_layout[0]);
            frame.buffer_mut().set_style(
                popup_layout[0],
                Style::default().bg(self.config.colorscheme.background),
            );
            frame.render_widget(popup, popup_layout[0]);
        }
    }
}