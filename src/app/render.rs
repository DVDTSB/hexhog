use crate::app::App;

impl App {
    pub fn render(&mut self, frame: &mut ratatui::Frame) {
        for window in self.windows.iter_mut() {
            if window.is_open() {
                window.render(frame);
            }
        }

        self.render_status_bar(frame);
    }

    pub fn render_status_bar(&mut self, frame: &mut ratatui::Frame) {
        let area = ratatui::layout::Rect {
            x: 0,
            y: frame.area().height - 1,
            width: frame.area().width,
            height: 1,
        };

        let status_text = format!(
            "h-help | cursor: {} bytes | size: {} bytes",
            0,
            0,
        );

        let paragraph = ratatui::widgets::Paragraph::new(status_text)
            .style(ratatui::style::Style::default().bg(self.config.colorscheme.primary).fg(self.config.colorscheme.background)).centered();
        frame.render_widget(paragraph, area);
    }
}