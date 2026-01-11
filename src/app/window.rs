use ratatui::widgets::{Clear, Padding};

pub trait Window {
    fn is_open(&self) -> bool;
    fn set_open(&mut self, open: bool);
    fn set_focus(&mut self, focused: bool);
    fn render(&mut self, frame: &mut ratatui::Frame);
    fn handle_event(&mut self, event: crossterm::event::Event) -> color_eyre::Result<()>;
}

pub fn get_popup_area(
    screen_rect: ratatui::layout::Rect,
    popup_width: u16,
    popup_height: u16,
    horizontal_aligment: f32,
    vertical_aligment: f32,
) -> ratatui::layout::Rect {
    let screen_width = screen_rect.width;
    let screen_height = screen_rect.height;

    let x = screen_rect.x + ((screen_width - popup_width) as f32 * horizontal_aligment) as u16;
    let y = screen_rect.y + ((screen_height - popup_height) as f32 * vertical_aligment) as u16;
    ratatui::layout::Rect {
        x,
        y,
        width: popup_width,
        height: popup_height,
    }
}
pub struct TextWindow {
    pub title: String,
    pub text: String,
    pub is_open: bool,
    pub is_focused: bool,
}

impl Window for TextWindow {
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
        let layout = get_popup_area(frame.area(), 32, 10, 0.5, 0.5);

        let block = ratatui::widgets::Block::default()
            .title(format!(" {} ", self.title))
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(if self.is_focused {
                ratatui::style::Style::default().fg(ratatui::style::Color::LightBlue)
            } else {
                ratatui::style::Style::default()
            })
            .padding(Padding::symmetric(4, 1));
        let paragraph = ratatui::widgets::Paragraph::new(self.text.clone())
            .wrap(ratatui::widgets::Wrap { trim: true }).alignment(ratatui::layout::Alignment::Center)
            .block(block);
        frame.render_widget(Clear, layout);
        frame.render_widget(paragraph, layout);
    }
    fn handle_event(&mut self, event: crossterm::event::Event) -> color_eyre::Result<()> {
        // Event handling logic here
        Ok(())
    }
}
