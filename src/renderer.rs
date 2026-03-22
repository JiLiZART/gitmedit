use crate::app::App;
use crate::document::ContentLine;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::layout::Alignment;
use crate::context::GitContext;

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

        // Render help overlay on top if visible.
        if app.is_help_visible() {
            Self::render_help_overlay(frame, app);
        }
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

        // Full terminal width — the content area spans the entire frame width.
        let visible_width = area.width as usize;

        // Compute vertical scroll offset: keep cursor visible in the content area.
        let visible_height = area.height as usize;
        let scroll_top = if cursor_full_row != usize::MAX && cursor_full_row >= visible_height {
            cursor_full_row - visible_height + 1
        } else {
            0
        };

        // Compute horizontal scroll offset: keep cursor visible within full terminal width.
        // scroll_left is the number of columns scrolled off the left edge.
        let scroll_left = if visible_width > 0 && cursor_col >= visible_width {
            // Keep cursor at the rightmost column of the visible area.
            cursor_col - visible_width + 1
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
                    let raw_text = if editable_idx < textarea_lines.len() {
                        textarea_lines[editable_idx].as_str()
                    } else {
                        ""
                    };
                    // Apply horizontal scroll: skip `scroll_left` chars, then take `visible_width`.
                    let text = Self::scroll_line(raw_text, scroll_left, visible_width);
                    let is_cursor_line = full_idx == cursor_full_row;

                    if is_cursor_line {
                        // Highlight the cursor line with an underline modifier.
                        let style = Style::default().add_modifier(Modifier::UNDERLINED);
                        styled_lines.push(Line::styled(text, style));
                    } else {
                        styled_lines.push(Line::raw(text));
                    }
                    editable_idx += 1;
                }
                ContentLine::Comment(s) => {
                    let text = Self::scroll_line(s, scroll_left, visible_width);
                    styled_lines.push(Line::from(Span::styled(
                        text,
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                ContentLine::ConflictMarker(s) => {
                    let text = Self::scroll_line(s, scroll_left, visible_width);
                    styled_lines.push(Line::from(Span::styled(
                        text,
                        Style::default().bg(Color::Red).fg(Color::White),
                    )));
                }
            }
        }

        // No wrapping: lines are explicitly truncated to visible_width above.
        let content_widget = Paragraph::new(Text::from(styled_lines));
        frame.render_widget(content_widget, area);

        // Position the blinking terminal cursor at the correct cell.
        // The visual column is cursor_col offset by scroll_left, clamped to visible_width.
        if app.document().editable_count() > 0
            && cursor_full_row != usize::MAX
            && cursor_full_row >= scroll_top
        {
            let visual_row = cursor_full_row - scroll_top;
            let visual_col = cursor_col.saturating_sub(scroll_left);
            if visual_row < visible_height && visual_col < visible_width {
                frame.set_cursor_position((
                    area.x + visual_col as u16,
                    area.y + visual_row as u16,
                ));
            }
        }
    }

    /// Scroll a line horizontally: skip `offset` chars then take up to `width` chars.
    /// Returns a String that fits within `width` columns.
    fn scroll_line(s: &str, offset: usize, width: usize) -> String {
        if width == 0 {
            return String::new();
        }
        // Use char indices to handle multi-byte characters correctly.
        let chars: Vec<char> = s.chars().collect();
        let start = offset.min(chars.len());
        let end = (start + width).min(chars.len());
        chars[start..end].iter().collect()
    }

    /// Compute a centered rectangle of (percent_x% wide, percent_y% tall) within r.
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1]);

        horizontal[1]
    }

    /// Return mode-aware help lines for the current git context.
    fn help_text_for_context(context: &GitContext) -> Vec<Line<'static>> {
        let base_actions = vec![
            "Ctrl+S  Save message",
            "Esc     Cancel (discard)",
            "",
            "Ctrl+C  Copy selection",
            "Ctrl+X  Cut selection",
            "Ctrl+V  Paste",
            "",
            "Ctrl+U  Delete line",
            "Ctrl+Z  Undo",
            "Ctrl+Y  Redo",
            "Ctrl+W  Delete word",
            "Ctrl+D  Delete next word",
        ];

        let mut lines: Vec<Line<'static>> = base_actions
            .iter()
            .map(|s| Line::raw(*s))
            .collect();

        match context {
            GitContext::Merge => {
                lines.push(Line::raw(""));
                lines.push(Line::raw("NOTE: Conflict markers (<<<, ===, >>>) are read-only."));
            }
            _ => {}
        }

        lines
    }

    /// Render a centered help overlay modal on top of the existing content.
    fn render_help_overlay(frame: &mut Frame, app: &App) {
        let popup_area = Self::centered_rect(60, 70, frame.area());

        let help_text = Self::help_text_for_context(app.context());

        let help_widget = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Help (Esc to close) ")
            )
            .style(Style::default().bg(Color::DarkGray).fg(Color::White))
            .alignment(Alignment::Left);

        frame.render_widget(help_widget, popup_area);
    }

    fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
        let first_line = app.document().first_line();
        let char_count = first_line.chars().count();
        let counter_color = if char_count <= 50 {
            Color::Green
        } else if char_count <= 72 {
            Color::Yellow
        } else {
            Color::Red
        };
        let counter_text = format!("Chars: {}", char_count);
        let counter_span = Span::styled(
            counter_text,
            Style::default().fg(counter_color).add_modifier(Modifier::BOLD),
        );

        let has_blank = app.document().has_blank_line_after_subject();
        let blank_warning = if !has_blank {
            Span::styled(" [No blank line]", Style::default().fg(Color::Yellow))
        } else {
            Span::raw("")
        };

        let actions_span = Span::raw("  |  ^S Save  Esc Cancel  ^H Help");

        let status_line = Line::from(vec![counter_span, blank_warning, actions_span]);
        let status_widget = Paragraph::new(status_line)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));
        frame.render_widget(status_widget, area);
    }
}
