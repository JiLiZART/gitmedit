use crate::app::App;
use crate::document::ContentLine;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Wrap};

/// Stateless renderer with per-line styling for Content, Comment, and ConflictMarker lines.
pub struct Renderer;

impl Renderer {
    pub fn render(frame: &mut Frame, app: &App) {
        // Split into content area (everything above) and 1-line status bar.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(frame.area());

        Self::render_content(frame, app, chunks[0]);
        Self::render_status_bar(frame, app, chunks[1]);
    }

    fn render_content(frame: &mut Frame, app: &App, area: Rect) {
        let doc_lines = app.document().lines();
        let (cursor_row, cursor_col) = app.textarea().cursor();
        let textarea_lines = app.textarea().lines();

        // Compute which full-document row the textarea cursor is on.
        let cursor_full_row = if app.document().editable_count() > 0 {
            app.document().full_row_for_editable(cursor_row)
        } else {
            usize::MAX // no editable lines — no cursor
        };

        // Compute scroll offset: keep cursor visible in the content area.
        let visible_height = area.height as usize;
        let scroll_top = if cursor_full_row != usize::MAX && cursor_full_row >= visible_height {
            cursor_full_row - visible_height + 1
        } else {
            0
        };

        let mut styled_lines: Vec<Line> = Vec::new();
        let mut editable_idx: usize = 0;

        for (full_idx, line) in doc_lines.iter().enumerate() {
            // Count editable lines that fall before the scroll viewport.
            if full_idx < scroll_top {
                if matches!(line, ContentLine::Content(_)) {
                    editable_idx += 1;
                }
                continue;
            }
            // Stop once we've filled the visible area.
            if styled_lines.len() >= visible_height {
                break;
            }

            match line {
                ContentLine::Content(_) => {
                    let text = if editable_idx < textarea_lines.len() {
                        textarea_lines[editable_idx].as_str()
                    } else {
                        ""
                    };
                    let is_cursor_line = full_idx == cursor_full_row;

                    if is_cursor_line {
                        // Highlight the cursor line with an underline modifier.
                        let style = Style::default().add_modifier(Modifier::UNDERLINED);
                        styled_lines.push(Line::styled(text.to_string(), style));
                    } else {
                        styled_lines.push(Line::raw(text.to_string()));
                    }
                    editable_idx += 1;
                }
                ContentLine::Comment(s) => {
                    styled_lines.push(Line::from(Span::styled(
                        s.clone(),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                ContentLine::ConflictMarker(s) => {
                    styled_lines.push(Line::from(Span::styled(
                        s.clone(),
                        Style::default().bg(Color::Red).fg(Color::White),
                    )));
                }
            }
        }

        let content_widget = Paragraph::new(Text::from(styled_lines))
            .wrap(Wrap { trim: false });
        frame.render_widget(content_widget, area);

        // Position the blinking terminal cursor at the correct cell.
        if app.document().editable_count() > 0
            && cursor_full_row != usize::MAX
            && cursor_full_row >= scroll_top
        {
            let visual_row = cursor_full_row - scroll_top;
            if visual_row < visible_height {
                frame.set_cursor_position((
                    area.x + cursor_col as u16,
                    area.y + visual_row as u16,
                ));
            }
        }
    }

    fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
        let status_text = format!("^S Save  Esc Cancel  [{:?}]", app.context());
        let status_widget = Paragraph::new(status_text)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));
        frame.render_widget(status_widget, area);
    }
}
