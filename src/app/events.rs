use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyModifiers};

use crate::app::App;

impl App{
    pub fn handle_events(&mut self) -> Result<()> {
        while crossterm::event::poll(std::time::Duration::from_millis(0))? {
            let event = crossterm::event::read()?;
            
            if let Some(window) = self.windows.get_mut(self.focus_index) {
                window.handle_event(event)?;
            }
        }
        Ok(())
    }
    pub fn handle_app_event(&mut self, event: crossterm::event::Event) -> Result<()> {
        match event {
            crossterm::event::Event::Key(key_event) => match key_event {
                crossterm::event::KeyEvent {code: KeyCode::Char('q'), modifiers: KeyModifiers::CONTROL, kind: _, state: _} => {
                    self.running = false;
                }
                _ => {}
            }
            _ => {}
        }
        Ok(())
    }
}