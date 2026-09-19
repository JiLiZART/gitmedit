use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table};
use ratatui_textarea::TextArea;

use crate::layout::{self, Pane};
use crate::rebase::{self, TodoLine};
use crate::session::{App, Body, RebaseBody};
use crate::{status, wrap};

const ACTION_W: u16 = 7;
const HASH_W: u16 = 8;

type Selection = Option<((usize, usize), (usize, usize))>;

/// Viewport state a pane drawing hands back to the app.
enum Drawn {
    Editor { top: usize, width: usize },
    Table { top: usize },
}

pub fn render(frame: &mut Frame, app: &mut App) {
    let rects = layout::compute(frame.area(), app.has_right(), app.focus);
    app.rects = rects;
    if let Some(area) = rects.left {
        render_left(frame, app, area);
    }
    if let Some(area) = rects.right {
        render_right(frame, app, area);
    }
    let entries = key_bar_entries(app);
    frame.render_widget(
        Paragraph::new(key_bar_line(&entries, rects.key_bar.width as usize))
            .style(Style::default().fg(Color::White).bg(Color::DarkGray)),
        rects.key_bar,
    );
    if app.show_help {
        render_help(frame, app);
    }
}

fn pane_block(title: String, focused: bool) -> Block<'static> {
    let color = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(title)
}

fn render_left(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Pane::Left;
    let show_cursor = focused && !app.show_help;
    let file_title = format!(" {} ", app.file_name);
    let drawn = match &app.body {
        Body::Message(m) => draw_editor(
            frame,
            area,
            pane_block(file_title, focused),
            &m.editor,
            app.editor_top,
            show_cursor,
        ),
        Body::Plain(editor) => draw_editor(
            frame,
            area,
            pane_block(file_title, focused),
            editor,
            app.editor_top,
            show_cursor,
        ),
        Body::Rebase(r) => match &r.raw {
            Some(raw) => draw_editor(
                frame,
                area,
                pane_block(" Rebase todo (raw) ".into(), focused),
                raw,
                app.editor_top,
                show_cursor,
            ),
            None => {
                let title = match r.todo.range(app.comment_char) {
                    Some(range) => format!(" Rebase {range} "),
                    None => " Rebase ".to_string(),
                };
                draw_table(
                    frame,
                    area,
                    pane_block(title, focused),
                    r,
                    app.table_top,
                    show_cursor,
                )
            }
        },
    };
    match drawn {
        Drawn::Editor { top, width } => {
            app.editor_top = top;
            app.editor_width = width;
        }
        Drawn::Table { top } => app.table_top = top,
    }
}

/// Draw a soft-wrapped text editor, keeping the cursor row in view.
fn draw_editor(
    frame: &mut Frame,
    area: Rect,
    block: Block<'static>,
    editor: &TextArea<'static>,
    top: usize,
    show_cursor: bool,
) -> Drawn {
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let width = inner.width as usize;
    let height = inner.height as usize;
    let (cursor_row, cursor_col) = editor.cursor();
    let selection = editor.selection_range();

    let mut rows: Vec<Line<'static>> = Vec::new();
    let mut cursor_at = (0, 0);
    for (index, line) in editor.lines().iter().enumerate() {
        let segments = wrap::wrap(line, width);
        if index == cursor_row {
            let (row, col) = wrap::locate(&segments, cursor_col);
            cursor_at = (rows.len() + row, col);
        }
        let base = if is_conflict_marker(line) {
            Style::default().fg(Color::White).bg(Color::Red)
        } else {
            Style::default()
        };
        rows.extend(
            segments
                .iter()
                .map(|seg| segment_line(seg, index, selection, base)),
        );
    }

    let mut top = top;
    if cursor_at.0 < top {
        top = cursor_at.0;
    }
    if height > 0 && cursor_at.0 >= top + height {
        top = cursor_at.0 + 1 - height;
    }
    let visible: Vec<Line<'static>> = rows.into_iter().skip(top).take(height).collect();
    frame.render_widget(Paragraph::new(visible), inner);

    if show_cursor && width > 0 && height > 0 {
        // ponytail: a cursor just past a completely full row is drawn on its last column
        let col = cursor_at.1.min(width - 1);
        frame.set_cursor_position((inner.x + col as u16, inner.y + (cursor_at.0 - top) as u16));
    }
    Drawn::Editor { top, width }
}

fn segment_line(
    seg: &wrap::Segment,
    line: usize,
    selection: Selection,
    base: Style,
) -> Line<'static> {
    let chars: Vec<char> = seg.text.chars().collect();
    let (from, to) = match selection {
        Some(((start_row, start_col), (end_row, end_col)))
            if start_row <= line && line <= end_row =>
        {
            let start = if start_row == line { start_col } else { 0 };
            let end = if end_row == line { end_col } else { usize::MAX };
            (
                start.saturating_sub(seg.start).min(chars.len()),
                end.saturating_sub(seg.start).min(chars.len()),
            )
        }
        _ => (0, 0),
    };
    if from >= to {
        return Line::from(Span::styled(seg.text.clone(), base));
    }
    let part = |a: usize, b: usize| chars[a..b].iter().collect::<String>();
    Line::from(vec![
        Span::styled(part(0, from), base),
        Span::styled(part(from, to), base.add_modifier(Modifier::REVERSED)),
        Span::styled(part(to, chars.len()), base),
    ])
}

fn is_conflict_marker(line: &str) -> bool {
    ['<', '=', '>'].iter().any(|&marker| {
        let run = line.chars().take_while(|&c| c == marker).count();
        run == 6 || run == 7
    })
}

struct TableRow {
    line: usize,
    action: String,
    style: Style,
    hash: String,
    subject: Vec<String>,
    dim: bool,
}

fn wrapped(text: &str, width: usize) -> Vec<String> {
    wrap::wrap(text, width)
        .into_iter()
        .map(|s| s.text)
        .collect()
}

fn action_style(action: rebase::Action) -> Style {
    let color = match action {
        rebase::Action::Pick => Color::Green,
        rebase::Action::Reword => Color::Blue,
        rebase::Action::Edit => Color::Magenta,
        rebase::Action::Squash => Color::Yellow,
        rebase::Action::Fixup => Color::Cyan,
        rebase::Action::Drop => Color::Red,
    };
    Style::default().fg(color)
}

fn draw_table(
    frame: &mut Frame,
    area: Rect,
    block: Block<'static>,
    r: &RebaseBody,
    top: usize,
    show_cursor: bool,
) -> Drawn {
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let subject_w = inner.width.saturating_sub(ACTION_W + HASH_W + 2) as usize;
    let height = inner.height as usize;
    let selected_line = r.selected_line();

    let rows: Vec<TableRow> = r
        .todo
        .lines
        .iter()
        .enumerate()
        .filter_map(|(line, l)| match l {
            TodoLine::Commit(c) => {
                let subject = if r.rewords.contains_key(&c.hash) {
                    format!("✎ {}", r.display_subject(c))
                } else {
                    c.subject().to_string()
                };
                Some(TableRow {
                    line,
                    action: c.action.as_str().to_string(),
                    style: action_style(c.action),
                    hash: c.hash.chars().take(7).collect(),
                    subject: wrapped(&subject, subject_w),
                    dim: false,
                })
            }
            TodoLine::Other(s) => {
                let (word, rest) = s
                    .trim_start()
                    .split_once(' ')
                    .unwrap_or((s.trim_start(), ""));
                Some(TableRow {
                    line,
                    action: word.to_string(),
                    style: Style::default().fg(Color::Magenta),
                    hash: String::new(),
                    subject: wrapped(rest, subject_w),
                    dim: false,
                })
            }
            TodoLine::Unknown(s) => Some(TableRow {
                line,
                action: String::new(),
                style: Style::default(),
                hash: String::new(),
                subject: wrapped(s, subject_w),
                dim: true,
            }),
            TodoLine::Comment(_) | TodoLine::Blank(_) => None,
        })
        .collect();

    let selected_row = rows.iter().position(|row| Some(row.line) == selected_line);
    let mut top = top.min(rows.len().saturating_sub(1));
    if let Some(s) = selected_row {
        if s < top {
            top = s;
        }
        while top < s
            && rows[top..=s]
                .iter()
                .map(|row| row.subject.len())
                .sum::<usize>()
                > height
        {
            top += 1;
        }
    }

    let table_rows: Vec<Row<'static>> = rows
        .iter()
        .skip(top)
        .map(|row| {
            let selected = Some(row.line) == selected_line;
            let subject = match (&r.inline, selected) {
                (Some(editor), true) => vec![editor.lines().join(" ")],
                _ => row.subject.clone(),
            };
            let mut style = if row.dim {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };
            if selected {
                style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
            }
            Row::new(vec![
                Cell::from(row.action.clone()).style(row.style),
                Cell::from(row.hash.clone()),
                Cell::from(subject.join("\n")),
            ])
            .height(subject.len().max(1) as u16)
            .style(style)
        })
        .collect();
    let widths = [
        Constraint::Length(ACTION_W),
        Constraint::Length(HASH_W),
        Constraint::Min(0),
    ];
    frame.render_widget(Table::new(table_rows, widths), inner);

    if let (Some(editor), Some(s), true) = (&r.inline, selected_row, show_cursor) {
        let y: usize = rows[top..s].iter().map(|row| row.subject.len()).sum();
        let x = inner.x + ACTION_W + HASH_W + 2 + editor.cursor().1 as u16;
        if y < height {
            frame.set_cursor_position((x.min(inner.right().saturating_sub(1)), inner.y + y as u16));
        }
    }
    Drawn::Table { top }
}

fn render_right(frame: &mut Frame, app: &mut App, area: Rect) {
    let width = area.width.saturating_sub(2) as usize;
    let (title, lines) = match &app.body {
        Body::Message(m) => (
            format!(" {} ", m.status.branch.as_deref().unwrap_or("Status")),
            status::render_lines(&m.status, width),
        ),
        Body::Rebase(r) => (" Details ".to_string(), details_lines(r, width)),
        Body::Plain(_) => return,
    };
    let block = pane_block(title, app.focus == Pane::Right);
    let height = block.inner(area).height as usize;
    app.right_scroll = app.right_scroll.min(lines.len().saturating_sub(height));
    app.right_page = height;
    let top = app.right_scroll;
    let block = if lines.len() > height {
        let bottom = (top + height).min(lines.len());
        block.title_bottom(
            Line::from(format!(" {}-{}/{} ", top + 1, bottom, lines.len())).right_aligned(),
        )
    } else {
        block
    };
    let visible: Vec<Line<'static>> = lines.into_iter().skip(top).take(height).collect();
    frame.render_widget(Paragraph::new(visible).block(block), area);
}

fn details_lines(r: &RebaseBody, width: usize) -> Vec<Line<'static>> {
    let dim = Style::default().fg(Color::DarkGray);
    let s = r.todo.summary();
    let mut out = vec![
        Line::from(Span::styled(
            format!("{} → {} commits", s.total, s.result),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "{} squash/fixup · {} drop · {} reword",
            s.squash_fixup, s.drop, s.reword
        )),
    ];
    if s.first_is_squash {
        out.push(Line::from(Span::styled(
            "⚠ the first commit cannot be squashed or fixed up",
            Style::default().fg(Color::Red),
        )));
    }
    out.push(Line::default());
    let Some(line) = r.selected_line() else {
        return out;
    };
    match &r.todo.lines[line] {
        TodoLine::Commit(c) => {
            out.push(Line::from(Span::styled(
                c.hash.clone(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            match r.details.get(&c.hash) {
                None => out.push(Line::from(Span::styled("loading…", dim))),
                Some(None) => out.push(Line::from(Span::styled("details unavailable", dim))),
                Some(Some(details)) => {
                    for message_line in &details.message {
                        out.extend(wrapped(message_line, width).into_iter().map(Line::from));
                    }
                    out.push(Line::default());
                    for (badge, path) in &details.files {
                        out.push(Line::from(vec![
                            Span::styled(badge.to_string(), status::badge_style(*badge)),
                            Span::raw(" "),
                            Span::raw(path.clone()),
                        ]));
                    }
                }
            }
        }
        TodoLine::Other(s) => out.push(Line::from(s.clone())),
        _ => {}
    }
    out
}

fn key_bar_entries(app: &App) -> Vec<(&'static str, &'static str)> {
    if app.show_help {
        return vec![("Esc", "Close"), ("↑↓", "Scroll")];
    }
    let mut keys = vec![("^S", "Save")];
    if let Body::Rebase(r) = &app.body
        && r.inline.is_some()
    {
        keys.extend([("Enter", "Confirm"), ("Esc", "Discard"), ("^H", "Help")]);
        return keys;
    }
    if app.focus == Pane::Right {
        keys.extend([
            ("Esc", "Back"),
            ("^H", "Help"),
            ("↑↓", "Scroll"),
            ("PgUp/PgDn", "Page"),
            ("Home/End", "Top/End"),
        ]);
        return keys;
    }
    keys.extend([("Esc", "Cancel"), ("^H", "Help")]);
    match &app.body {
        Body::Rebase(r) if r.raw.is_none() => keys.extend([
            ("p r e s f d", "Action"),
            ("Tab", "Cycle"),
            ("Alt+↑↓", "Move"),
            ("Enter", "Reword"),
            ("^E", "Raw"),
            ("Alt+→", "Details"),
        ]),
        Body::Rebase(_) => keys.push(("^E", "Table")),
        _ if app.has_right() => keys.extend([("Alt+→", "Status"), ("^T", "Switch pane")]),
        _ => {}
    }
    keys
}

/// Key bar text; entries that do not fit the width are dropped from the end.
fn key_bar_line(entries: &[(&str, &str)], width: usize) -> Line<'static> {
    let mut spans = Vec::new();
    let mut used = 0;
    for (key, label) in entries {
        let len = key.chars().count() + label.chars().count() + 3;
        if used + len > width {
            break;
        }
        spans.push(Span::styled(
            key.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(format!(" {label}  ")));
        used += len;
    }
    Line::from(spans)
}

fn help_lines(app: &App) -> Vec<Line<'static>> {
    let mut lines: Vec<&'static str> = vec![
        "Global",
        "  Ctrl+S          Save and exit",
        "  Ctrl+H          Toggle this help",
        "  Alt+← / Alt+→   Focus left / right pane (or click a pane)",
        "  Ctrl+T          Switch pane (narrow terminals show one pane)",
        "  Esc             Cancel in the left pane, back to it from the right",
        "",
    ];
    match &app.body {
        Body::Rebase(_) => lines.extend([
            "Rebase table",
            "  ↑/↓ Home/End    Select instruction",
            "  p r e s f d     Pick / reword / edit / squash / fixup / drop",
            "  Tab             Cycle pick → squash → fixup → drop",
            "  Alt+↑ / Alt+↓   Move instruction",
            "  Enter           Reword subject inline (Enter confirm, Esc discard)",
            "  Ctrl+E          Toggle raw text editing",
            "",
            "Details pane",
            "  ↑/↓ PgUp/PgDn Home/End   Scroll (or mouse wheel)",
            "",
            "Inline reword needs gitmedit as core.editor as well.",
            "",
            "Commands",
            "  pick        use commit",
            "  reword      use commit, but edit the commit message",
            "  edit        use commit, but stop for amending",
            "  squash      use commit, but meld into previous commit",
            "  fixup       like squash, but keep only the previous message",
            "  exec        run command (the rest of the line) using shell",
            "  break       stop here (continue with git rebase --continue)",
            "  drop        remove commit",
            "  label       label current HEAD with a name",
            "  reset       reset HEAD to a label",
            "  merge       create a merge commit",
            "  update-ref  track a placeholder for a ref to update",
        ]),
        _ => lines.extend([
            "Editor",
            "  Ctrl+C / Ctrl+X / Ctrl+V   Copy / cut / paste",
            "  Ctrl+U                     Delete line",
            "  Ctrl+W / Ctrl+D            Delete previous / next word",
            "  Ctrl+Z / Ctrl+Y            Undo / redo",
            "",
            "Status pane",
            "  ↑/↓ PgUp/PgDn Home/End     Scroll (or mouse wheel)",
        ]),
    }
    lines.into_iter().map(Line::from).collect()
}

fn render_help(frame: &mut Frame, app: &mut App) {
    let area = centered(frame.area(), 70, 80);
    let lines = help_lines(app);
    let height = area.height.saturating_sub(2) as usize;
    app.help_scroll = app.help_scroll.min(lines.len().saturating_sub(height));
    let visible: Vec<Line<'static>> = lines.into_iter().skip(app.help_scroll).collect();
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(visible).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help (Esc to close) "),
        ),
        area,
    );
}

fn centered(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let width = (area.width as u32 * percent_x as u32 / 100) as u16;
    let height = (area.height as u32 * percent_y as u32 / 100) as u16;
    Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::GitContext;
    use crate::session::{Action, ScrollBy};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    const AMMEND2: &str = include_str!("../fixtures/ammend2_fixture.txt");
    const MERGE2: &str = include_str!("../fixtures/merge_fixture2.txt");
    const TODO: &str = include_str!("../fixtures/squash_fixture.txt");

    fn draw(app: &mut App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| render(frame, app)).unwrap();
        let buffer = terminal.backend().buffer();
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn text(lines: &[Line]) -> String {
        lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn commit_app(raw: &str) -> App {
        let mut app = App::new(raw, GitContext::Commit, None, '#');
        app.file_name = "COMMIT_EDITMSG".into();
        app
    }

    #[test]
    fn message_layout_shows_message_and_status() {
        let screen = draw(&mut commit_app(AMMEND2), 120, 30);
        assert!(screen.contains("fix: release volume"));
        assert!(screen.contains("Staged (6)"));
        assert!(screen.contains("COMMIT_EDITMSG"));
        assert!(screen.contains("TASK-1111-fix-stage-view"));
        assert!(!screen.contains("Please enter"));
    }

    #[test]
    fn narrow_terminal_shows_one_pane_and_toggle_swaps() {
        let mut app = commit_app(AMMEND2);
        let screen = draw(&mut app, 80, 30);
        assert!(screen.contains("fix: release volume") && !screen.contains("Staged"));
        app.apply(Action::ToggleFocus);
        let screen = draw(&mut app, 80, 30);
        assert!(screen.contains("Staged (6)") && !screen.contains("fix: release volume"));
    }

    #[test]
    fn soft_wrap_shows_the_whole_long_line() {
        let long: String = (0..40)
            .map(|i| format!("word{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let mut app = App::new(&format!("{long}\n"), GitContext::Unknown, None, '#');
        let screen = draw(&mut app, 60, 20);
        assert!(screen.contains("word0") && screen.contains("word39"));
        assert_eq!(app.editor_width, 58);
    }

    #[test]
    fn conflict_markers_are_highlighted() {
        let mut app = App::new("<<<<<<< HEAD\nours\n", GitContext::Unknown, None, '#');
        let mut terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
        terminal.draw(|frame| render(frame, &mut app)).unwrap();
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(1, 1)].bg, Color::Red);
        assert_ne!(buffer[(1, 2)].bg, Color::Red);
    }

    #[test]
    fn rebase_layout_shows_table_and_summary() {
        let mut app = App::new(TODO, GitContext::Rebase, None, '#');
        let screen = draw(&mut app, 140, 30);
        assert!(screen.contains("Rebase 36d7eda..aa619f8"));
        assert!(screen.contains("54763e6"));
        assert!(screen.contains("14 → 14 commits"));
        assert!(!screen.contains("# Commands:"));
    }

    #[test]
    fn right_scroll_clamps_to_the_last_page() {
        let mut app = App::new(MERGE2, GitContext::Merge, None, '#');
        app.apply(Action::ScrollRight(ScrollBy::End));
        let screen = draw(&mut app, 120, 30);
        assert!(screen.contains("sync-ui"));
        assert!(app.right_scroll < usize::MAX);
        assert!(app.right_scroll > 0);
    }

    #[test]
    fn key_bar_drops_entries_that_do_not_fit() {
        let entries = [
            ("^S", "Save"),
            ("Esc", "Cancel"),
            ("^H", "Help"),
            ("Tab", "Cycle"),
        ];
        let line = text(&[key_bar_line(&entries, 30)]);
        assert!(line.contains("Help") && !line.contains("Cycle"));
    }

    #[test]
    fn help_lists_keys_for_the_layout() {
        let rebase_help = text(&help_lines(&App::new(TODO, GitContext::Rebase, None, '#')));
        assert!(rebase_help.contains("update-ref") && rebase_help.contains("Ctrl+E"));
        let message_help = text(&help_lines(&commit_app(AMMEND2)));
        assert!(message_help.contains("Ctrl+C") && !message_help.contains("update-ref"));
    }
}
