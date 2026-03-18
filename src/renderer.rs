use crate::app::App;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::Paragraph;

/// Stateless renderer for Phase 1.
pub struct Renderer;

impl Renderer {
    pub fn render(frame: &mut Frame, app: &App) {
        // Split into content area (everything above) and 1-line status bar.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(frame.area());

        // Content area: file text split into lines.
        let lines: Vec<Line> = app.content().split('\n').map(Line::raw).collect();
        let content_widget = Paragraph::new(Text::from(lines));
        frame.render_widget(content_widget, chunks[0]);

        // Status bar: hotkey hints + context label.
        let status_text = format!("^S Save  Esc Cancel  [{:?}]", app.context());
        let status_widget = Paragraph::new(status_text)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));
        frame.render_widget(status_widget, chunks[1]);
    }
}
