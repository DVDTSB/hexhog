use crate::app::{App, change::Change, state::AppState};
use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
impl App {
    pub     fn handle_crossterm_events(&mut self) -> Result<()> {
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
                    self.set_idx(self.selection_start + self.clipboard.len() - 1);
                }

                (_, KeyCode::Backspace) => {
                    let idx = self.get_idx();
                    let (x, y) = self.selection_range();

                    //since cursor can also be outside data check this lol;
                    if x==y && y==self.data.len() {
                        self.move_left();
                        return ();
                    }

                    let old = self.data[x..(y + 1)].to_vec();

                    self.do_change(Change::Delete(idx, old));

                    //if where the cursor was now theres nothing then move it!
                    let new_idx = idx.min(self.data.len().saturating_sub(1));
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
}