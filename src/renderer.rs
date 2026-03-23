use crate::app::App;
use crate::document::{ContentLine, RebaseLine, RebaseAction};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
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

        match app.context() {
            GitContext::Rebase => {
                Self::render_rebase_table(frame, app, chunks[0]);
                Self::render_rebase_status_bar(frame, app, chunks[1]);
            }
            GitContext::Squash if !app.squash_log().is_empty() => {
                Self::render_squash_mode(frame, app, chunks[0]);
                Self::render_squash_status_bar(frame, app, chunks[1]);
            }
            _ => {
                Self::render_content(frame, app, chunks[0]);
                Self::render_status_bar(frame, app, chunks[1]);
            }
        }

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
            GitContext::Rebase => {
                // Rebase mode uses a completely different set of actions — no free-text editing.
                return vec![
                    Line::raw("Tab     Cycle action (pick/squash/fixup/drop)"),
                    Line::raw("Up/Down Navigate commits"),
                    Line::raw(""),
                    Line::raw("Ctrl+S  Save rebase plan"),
                    Line::raw("Esc     Cancel rebase"),
                    Line::raw(""),
                    Line::raw("NOTE: Comment lines are read-only."),
                    Line::raw("      Exec lines do not cycle."),
                ];
            }
            GitContext::Merge => {
                lines.push(Line::raw(""));
                lines.push(Line::raw("NOTE: Conflict markers (<<<, ===, >>>) are read-only."));
            }
            GitContext::Squash => {
                lines.push(Line::raw(""));
                lines.push(Line::raw("NOTE: Commit log above is read-only."));
                lines.push(Line::raw("      Edit the combined message below."));
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

    fn render_squash_mode(frame: &mut Frame, app: &App, area: Rect) {
        let log_lines = app.squash_log();
        // Log section: height based on number of log lines, capped at 40% of area.
        let log_height = (log_lines.len() as u16 + 2).min(area.height * 40 / 100);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(log_height),
                Constraint::Min(5),
            ])
            .split(area);

        let log_area = layout[0];
        let msg_area = layout[1];

        // Render read-only commit log with distinct background.
        let log_text: Vec<Line> = log_lines.iter().map(|line| {
            Line::from(Span::styled(
                line.clone(),
                Style::default().fg(Color::DarkGray),
            ))
        }).collect();

        let log_widget = Paragraph::new(log_text)
            .block(Block::default()
                .title(" Commit Log (read-only) ")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black)))
            .style(Style::default().fg(Color::DarkGray));

        frame.render_widget(log_widget, log_area);

        // Render editable message using the same render_content logic.
        Self::render_content(frame, app, msg_area);
    }

    fn render_squash_status_bar(frame: &mut Frame, app: &App, area: Rect) {
        let first_line = app.document().first_line();
        let char_count = first_line.chars().count();
        let counter_color = counter_color_for(char_count);
        let counter_text = format!("Chars: {}", char_count);
        let counter_span = Span::styled(
            counter_text,
            Style::default().fg(counter_color).add_modifier(Modifier::BOLD),
        );

        let has_blank = app.document().has_blank_line_after_subject();
        let blank_warning = blank_warning_span(has_blank);

        let actions_span = Span::raw("  |  ^S Save  Esc Cancel  ^H Help  [Squash]");

        let status_line = Line::from(vec![counter_span, blank_warning, actions_span]);
        let status_widget = Paragraph::new(status_line)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));
        frame.render_widget(status_widget, area);
    }

    fn render_rebase_table(frame: &mut Frame, app: &App, area: Rect) {
        let rebase_lines = app.rebase_lines();
        let selected_line_idx = app.selected_rebase_line_idx();

        // Compute scroll offset to keep selected row visible.
        let visible_height = area.height as usize;
        let scroll_offset = if let Some(idx) = selected_line_idx {
            if idx >= visible_height {
                idx.saturating_sub(visible_height / 2)
            } else {
                0
            }
        } else {
            0
        };

        let rows: Vec<Row> = rebase_lines
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_height)
            .map(|(i, line)| {
                let is_selected = selected_line_idx == Some(i);
                match line {
                    RebaseLine::Action { action, hash, subject } => {
                        let action_style = match action {
                            RebaseAction::Pick   => Style::default().fg(Color::Green),
                            RebaseAction::Squash => Style::default().fg(Color::Yellow),
                            RebaseAction::Fixup  => Style::default().fg(Color::Cyan),
                            RebaseAction::Drop   => Style::default().fg(Color::Red),
                            RebaseAction::Exec   => Style::default().fg(Color::Magenta),
                        };
                        let row_style = if is_selected {
                            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };
                        Row::new(vec![
                            Cell::from(format!("{:<7}", action.as_str())).style(action_style),
                            Cell::from(hash.chars().take(7).collect::<String>()),
                            Cell::from(subject.clone()),
                        ])
                        .style(row_style)
                    }
                    RebaseLine::Comment(text) => {
                        Row::new(vec![
                            Cell::from(text.clone()),
                        ])
                        .style(Style::default().fg(Color::DarkGray))
                    }
                }
            })
            .collect();

        let widths = [
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Min(20),
        ];

        let table = Table::new(rows, widths)
            .block(Block::default().title(" Rebase ").borders(Borders::NONE));

        frame.render_widget(table, area);
    }

    fn render_rebase_status_bar(frame: &mut Frame, app: &App, area: Rect) {
        let line_info = if !app.selectable_indices().is_empty() {
            format!("Line {}/{}", app.selected_rebase_idx() + 1, app.selectable_indices().len())
        } else {
            "No commits".to_string()
        };

        let status_line = Line::from(vec![
            Span::raw(line_info),
            Span::raw("  |  "),
            Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Cycle action  "),
            Span::styled("^S", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Save  "),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel  "),
            Span::styled("^H", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Help"),
        ]);

        let status_widget = Paragraph::new(status_line)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));
        frame.render_widget(status_widget, area);
    }

    fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
        let first_line = app.document().first_line();
        let char_count = first_line.chars().count();
        let counter_color = counter_color_for(char_count);
        let counter_text = format!("Chars: {}", char_count);
        let counter_span = Span::styled(
            counter_text,
            Style::default().fg(counter_color).add_modifier(Modifier::BOLD),
        );

        let has_blank = app.document().has_blank_line_after_subject();
        let blank_warning = blank_warning_span(has_blank);

        let actions_span = Span::raw("  |  ^S Save  Esc Cancel  ^H Help");

        let status_line = Line::from(vec![counter_span, blank_warning, actions_span]);
        let status_widget = Paragraph::new(status_line)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));
        frame.render_widget(status_widget, area);
    }
}

/// Return the appropriate counter color for a given character count:
/// - Green  if count <= 50
/// - Yellow if 51 <= count <= 72
/// - Red    if count >= 73
pub(crate) fn counter_color_for(count: usize) -> Color {
    if count <= 50 {
        Color::Green
    } else if count <= 72 {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// Return a blank-line warning span.
/// When `has_blank` is false, returns " [No blank line]" styled in yellow.
/// When `has_blank` is true, returns an empty raw span.
pub(crate) fn blank_warning_span(has_blank: bool) -> Span<'static> {
    if !has_blank {
        Span::styled(" [No blank line]", Style::default().fg(Color::Yellow))
    } else {
        Span::raw("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // counter_color_for
    // -------------------------------------------------------------------------

    #[test]
    fn test_counter_color_green_at_50() {
        assert_eq!(counter_color_for(50), Color::Green);
    }

    #[test]
    fn test_counter_color_yellow_at_60() {
        assert_eq!(counter_color_for(60), Color::Yellow);
    }

    #[test]
    fn test_counter_color_red_at_80() {
        assert_eq!(counter_color_for(80), Color::Red);
    }

    // Additional boundary checks
    #[test]
    fn test_counter_color_green_at_0() {
        assert_eq!(counter_color_for(0), Color::Green);
    }

    #[test]
    fn test_counter_color_yellow_at_51() {
        assert_eq!(counter_color_for(51), Color::Yellow);
    }

    #[test]
    fn test_counter_color_yellow_at_72() {
        assert_eq!(counter_color_for(72), Color::Yellow);
    }

    #[test]
    fn test_counter_color_red_at_73() {
        assert_eq!(counter_color_for(73), Color::Red);
    }

    // -------------------------------------------------------------------------
    // blank_warning_span
    // -------------------------------------------------------------------------

    #[test]
    fn test_blank_line_warning_not_shown_when_present() {
        let span = blank_warning_span(true);
        assert_eq!(span.content, "");
    }

    #[test]
    fn test_blank_line_warning_shown_when_missing() {
        let span = blank_warning_span(false);
        assert_eq!(span.content, " [No blank line]");
        assert_eq!(span.style.fg, Some(Color::Yellow));
    }
}
