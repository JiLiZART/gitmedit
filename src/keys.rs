use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use ratatui_textarea::Input;

use crate::layout::Pane;
use crate::rebase;
use crate::session::{Action, App, Body, ScrollBy};

/// Translate a terminal event into an action for the current state, or `None` to ignore it.
pub fn map_event(event: &Event, app: &App) -> Option<Action> {
    match event {
        Event::Mouse(m) => map_mouse(m.kind, m.column, m.row, app),
        Event::Key(key) if key.kind == KeyEventKind::Press => map_key(*key, app),
        _ => None,
    }
}

fn map_mouse(kind: MouseEventKind, column: u16, row: u16, app: &App) -> Option<Action> {
    let delta = match kind {
        MouseEventKind::ScrollUp => -1,
        MouseEventKind::ScrollDown => 1,
        MouseEventKind::Down(MouseButton::Left) if !app.show_help => return Some(Action::ClickAt { column, row }),
        _ => return None,
    };
    Some(if app.show_help { Action::ScrollHelp(delta) } else { Action::WheelAt { column, row, delta } })
}

fn map_key(key: KeyEvent, app: &App) -> Option<Action> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);

    if app.show_help {
        return match key.code {
            KeyCode::Esc => Some(Action::ToggleHelp),
            KeyCode::Char('h') if ctrl => Some(Action::ToggleHelp),
            KeyCode::Up => Some(Action::ScrollHelp(-1)),
            KeyCode::Down => Some(Action::ScrollHelp(1)),
            _ => None,
        };
    }
    match key.code {
        KeyCode::Char('s') if ctrl => return Some(Action::Save),
        KeyCode::Char('h') if ctrl => return Some(Action::ToggleHelp),
        _ => {}
    }

    let rebase = match &app.body {
        Body::Rebase(r) => Some(r),
        _ => None,
    };
    if rebase.is_some_and(|r| r.inline.is_some()) {
        return match key.code {
            KeyCode::Enter => Some(Action::CommitInline),
            KeyCode::Esc => Some(Action::CancelInline),
            _ => editing_key(key),
        };
    }

    // Many macOS terminals send ESC b / ESC f for Alt+Left / Alt+Right.
    match key.code {
        KeyCode::Left | KeyCode::Char('b') if alt => return Some(Action::FocusLeft),
        KeyCode::Right | KeyCode::Char('f') if alt => return Some(Action::FocusRight),
        KeyCode::Char('t') if ctrl => return Some(Action::ToggleFocus),
        _ => {}
    }

    if app.focus == Pane::Right {
        return match key.code {
            KeyCode::Esc => Some(Action::FocusLeft),
            KeyCode::Up => Some(Action::ScrollRight(ScrollBy::Lines(-1))),
            KeyCode::Down => Some(Action::ScrollRight(ScrollBy::Lines(1))),
            KeyCode::PageUp => Some(Action::ScrollRight(ScrollBy::Pages(-1))),
            KeyCode::PageDown => Some(Action::ScrollRight(ScrollBy::Pages(1))),
            KeyCode::Home => Some(Action::ScrollRight(ScrollBy::Top)),
            KeyCode::End => Some(Action::ScrollRight(ScrollBy::End)),
            _ => None,
        };
    }

    if key.code == KeyCode::Esc {
        return Some(Action::Cancel);
    }
    match rebase {
        Some(r) if r.raw.is_none() => table_key(key, ctrl, alt),
        Some(_) if ctrl && key.code == KeyCode::Char('e') => Some(Action::ToggleRaw),
        _ => match key.code {
            KeyCode::Up if key.modifiers.is_empty() => Some(Action::CursorUp),
            KeyCode::Down if key.modifiers.is_empty() => Some(Action::CursorDown),
            _ => editing_key(key),
        },
    }
}

fn table_key(key: KeyEvent, ctrl: bool, alt: bool) -> Option<Action> {
    match key.code {
        KeyCode::Up if alt => Some(Action::MoveInstruction { up: true }),
        KeyCode::Down if alt => Some(Action::MoveInstruction { up: false }),
        KeyCode::Up => Some(Action::SelectBy(ScrollBy::Lines(-1))),
        KeyCode::Down => Some(Action::SelectBy(ScrollBy::Lines(1))),
        KeyCode::PageUp => Some(Action::SelectBy(ScrollBy::Pages(-1))),
        KeyCode::PageDown => Some(Action::SelectBy(ScrollBy::Pages(1))),
        KeyCode::Home => Some(Action::SelectBy(ScrollBy::Top)),
        KeyCode::End => Some(Action::SelectBy(ScrollBy::End)),
        KeyCode::Tab => Some(Action::CycleInstruction),
        KeyCode::Enter => Some(Action::StartInline),
        KeyCode::Char('e') if ctrl => Some(Action::ToggleRaw),
        KeyCode::Char(c) if !ctrl && !alt => rebase::Action::from_key(c).map(Action::SetInstruction),
        _ => None,
    }
}

fn editing_key(key: KeyEvent) -> Option<Action> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    Some(match key.code {
        KeyCode::Char('u') if ctrl => Action::DeleteLine,
        KeyCode::Char('z') if ctrl => Action::Undo,
        KeyCode::Char('y') if ctrl => Action::Redo,
        KeyCode::Char('w') if ctrl => Action::DeleteWord,
        KeyCode::Char('d') if ctrl => Action::DeleteNextWord,
        KeyCode::Char('c') if ctrl => Action::Copy,
        KeyCode::Char('x') if ctrl => Action::Cut,
        KeyCode::Char('v') if ctrl => Action::Paste,
        _ => Action::Edit(Input::from(key)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::GitContext;
    use crossterm::event::MouseEvent;

    const AMMEND2: &str = include_str!("../fixtures/ammend2_fixture.txt");
    const TODO: &str = include_str!("../fixtures/squash_fixture.txt");

    fn message_app() -> App {
        App::new(AMMEND2, GitContext::Commit, None, '#')
    }

    fn rebase_app() -> App {
        App::new(TODO, GitContext::Rebase, None, '#')
    }

    fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new(code, modifiers))
    }

    fn plain(code: KeyCode) -> Event {
        key(code, KeyModifiers::NONE)
    }

    fn mouse(kind: MouseEventKind) -> Event {
        Event::Mouse(MouseEvent { kind, column: 7, row: 3, modifiers: KeyModifiers::NONE })
    }

    #[test]
    fn mouse_clicks_and_wheel() {
        let mut app = message_app();
        assert_eq!(map_event(&mouse(MouseEventKind::Down(MouseButton::Left)), &app), Some(Action::ClickAt { column: 7, row: 3 }));
        assert_eq!(map_event(&mouse(MouseEventKind::ScrollDown), &app), Some(Action::WheelAt { column: 7, row: 3, delta: 1 }));
        app.show_help = true;
        assert_eq!(map_event(&mouse(MouseEventKind::ScrollUp), &app), Some(Action::ScrollHelp(-1)));
        assert_eq!(map_event(&mouse(MouseEventKind::Down(MouseButton::Left)), &app), None);
    }

    #[test]
    fn key_release_is_ignored() {
        let mut release = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        release.kind = KeyEventKind::Release;
        assert_eq!(map_event(&Event::Key(release), &message_app()), None);
    }

    #[test]
    fn help_swallows_everything_but_close_and_scroll() {
        let mut app = message_app();
        app.show_help = true;
        assert_eq!(map_event(&plain(KeyCode::Char('a')), &app), None);
        assert_eq!(map_event(&key(KeyCode::Char('s'), KeyModifiers::CONTROL), &app), None);
        assert_eq!(map_event(&plain(KeyCode::Esc), &app), Some(Action::ToggleHelp));
        assert_eq!(map_event(&key(KeyCode::Char('h'), KeyModifiers::CONTROL), &app), Some(Action::ToggleHelp));
        assert_eq!(map_event(&plain(KeyCode::Down), &app), Some(Action::ScrollHelp(1)));
    }

    #[test]
    fn global_and_focus_keys() {
        let app = message_app();
        assert_eq!(map_event(&key(KeyCode::Char('s'), KeyModifiers::CONTROL), &app), Some(Action::Save));
        assert_eq!(map_event(&key(KeyCode::Char('h'), KeyModifiers::CONTROL), &app), Some(Action::ToggleHelp));
        assert_eq!(map_event(&key(KeyCode::Right, KeyModifiers::ALT), &app), Some(Action::FocusRight));
        assert_eq!(map_event(&key(KeyCode::Char('f'), KeyModifiers::ALT), &app), Some(Action::FocusRight));
        assert_eq!(map_event(&key(KeyCode::Left, KeyModifiers::ALT), &app), Some(Action::FocusLeft));
        assert_eq!(map_event(&key(KeyCode::Char('b'), KeyModifiers::ALT), &app), Some(Action::FocusLeft));
        assert_eq!(map_event(&key(KeyCode::Char('t'), KeyModifiers::CONTROL), &app), Some(Action::ToggleFocus));
        assert_eq!(map_event(&plain(KeyCode::Esc), &app), Some(Action::Cancel));
    }

    #[test]
    fn right_pane_keys_scroll_and_esc_returns() {
        let mut app = message_app();
        app.focus = Pane::Right;
        assert_eq!(map_event(&plain(KeyCode::Char('a')), &app), None);
        assert_eq!(map_event(&plain(KeyCode::Esc), &app), Some(Action::FocusLeft));
        assert_eq!(map_event(&plain(KeyCode::Up), &app), Some(Action::ScrollRight(ScrollBy::Lines(-1))));
        assert_eq!(map_event(&plain(KeyCode::PageDown), &app), Some(Action::ScrollRight(ScrollBy::Pages(1))));
        assert_eq!(map_event(&plain(KeyCode::Home), &app), Some(Action::ScrollRight(ScrollBy::Top)));
        assert_eq!(map_event(&plain(KeyCode::End), &app), Some(Action::ScrollRight(ScrollBy::End)));
    }

    #[test]
    fn message_editor_keys() {
        let app = message_app();
        assert_eq!(map_event(&plain(KeyCode::Up), &app), Some(Action::CursorUp));
        assert_eq!(map_event(&plain(KeyCode::Down), &app), Some(Action::CursorDown));
        assert_eq!(map_event(&key(KeyCode::Char('u'), KeyModifiers::CONTROL), &app), Some(Action::DeleteLine));
        assert_eq!(map_event(&key(KeyCode::Char('v'), KeyModifiers::CONTROL), &app), Some(Action::Paste));
        assert!(matches!(map_event(&plain(KeyCode::Char('a')), &app), Some(Action::Edit(_))));
    }

    #[test]
    fn rebase_table_keys() {
        let app = rebase_app();
        assert_eq!(map_event(&plain(KeyCode::Char('f')), &app), Some(Action::SetInstruction(rebase::Action::Fixup)));
        assert_eq!(map_event(&plain(KeyCode::Char('x')), &app), None);
        assert_eq!(map_event(&plain(KeyCode::Tab), &app), Some(Action::CycleInstruction));
        assert_eq!(map_event(&key(KeyCode::Up, KeyModifiers::ALT), &app), Some(Action::MoveInstruction { up: true }));
        assert_eq!(map_event(&plain(KeyCode::Down), &app), Some(Action::SelectBy(ScrollBy::Lines(1))));
        assert_eq!(map_event(&plain(KeyCode::End), &app), Some(Action::SelectBy(ScrollBy::End)));
        assert_eq!(map_event(&plain(KeyCode::Enter), &app), Some(Action::StartInline));
        assert_eq!(map_event(&key(KeyCode::Char('e'), KeyModifiers::CONTROL), &app), Some(Action::ToggleRaw));
        assert_eq!(map_event(&plain(KeyCode::Esc), &app), Some(Action::Cancel));
    }

    #[test]
    fn inline_editor_captures_keys() {
        let mut app = rebase_app();
        app.apply(Action::StartInline);
        assert_eq!(map_event(&plain(KeyCode::Enter), &app), Some(Action::CommitInline));
        assert_eq!(map_event(&plain(KeyCode::Esc), &app), Some(Action::CancelInline));
        assert!(matches!(map_event(&key(KeyCode::Right, KeyModifiers::ALT), &app), Some(Action::Edit(_))));
        assert!(matches!(map_event(&plain(KeyCode::Char('f')), &app), Some(Action::Edit(_))));
    }

    #[test]
    fn raw_mode_keys_edit_text_and_ctrl_e_returns() {
        let mut app = rebase_app();
        app.apply(Action::ToggleRaw);
        assert_eq!(map_event(&key(KeyCode::Char('e'), KeyModifiers::CONTROL), &app), Some(Action::ToggleRaw));
        assert!(matches!(map_event(&plain(KeyCode::Char('f')), &app), Some(Action::Edit(_))));
        assert_eq!(map_event(&plain(KeyCode::Down), &app), Some(Action::CursorDown));
    }
}
