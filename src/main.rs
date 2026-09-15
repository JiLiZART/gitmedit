use std::path::PathBuf;
use std::process;

use clap::Parser;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui_textarea::CursorMove;

use app::{Action, App, Outcome};
use context::GitContext;

mod app;
mod context;
mod details;
mod document;
mod layout;
mod message;
mod rebase;
mod renderer;
mod reword;
mod session;
mod status;
mod terminal;
mod wrap;
mod writer;

#[derive(Parser, Debug)]
#[command(name = "gitmedit", about = "Fast, distraction-free git editor")]
struct Cli {
    /// Path to the file to edit (provided by git)
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    // 1. Install panic hook FIRST — must precede any terminal state change.
    terminal::install_panic_hook();

    // 2. Parse argv[1].
    let cli = Cli::parse();
    let path = cli.path;

    // 3. Verify path exists.
    if !path.exists() {
        eprintln!("error: file not found: {:?}", path);
        process::exit(1);
    }

    // 4. Read file content.
    let raw_content = std::fs::read_to_string(&path)?;

    // 5. Detect git context.
    let ctx = context::detect_context(&path);

    // 6. Construct app state (parses Document + initialises TextArea).
    let mut app = App::new(&raw_content, ctx);

    // 7. Acquire raw-mode terminal guard.
    let mut guard = terminal::TerminalGuard::new()?;

    // 8. Event loop.
    loop {
        guard
            .terminal()
            .draw(|f| renderer::Renderer::render(f, &app))?;

        let event = crossterm::event::read()?;

        match &event {
            Event::Key(KeyEvent { code, modifiers, kind: KeyEventKind::Press, .. }) => {
                if app.is_help_visible() {
                    // Help overlay is visible: only Esc or Ctrl+H can dismiss it.
                    // All other input is blocked.
                    match (*code, *modifiers) {
                        (KeyCode::Esc, _) => {
                            app.apply(Action::DismissHelp);
                        }
                        (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                            app.apply(Action::DismissHelp);
                        }
                        _ => { /* ignore all other input while help visible */ }
                    }
                } else if *app.context() == GitContext::Rebase {
                    // Rebase mode: structured navigation, no free-text editing.
                    match (*code, *modifiers) {
                        // Ctrl+H — Show help overlay.
                        (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                            app.apply(Action::Help);
                        }
                        // Ctrl+S — Save rebase plan.
                        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                            match app.apply(Action::Save) {
                                Outcome::Save => {
                                    drop(guard);
                                    writer::FileWriter::write_atomic(&app.serialized_content(), &path)?;
                                    process::exit(0);
                                }
                                _ => {}
                            }
                        }
                        // Esc — Cancel rebase.
                        (KeyCode::Esc, _) => {
                            match app.apply(Action::Cancel) {
                                Outcome::Cancel => {
                                    drop(guard);
                                    process::exit(1);
                                }
                                _ => {}
                            }
                        }
                        // Tab — Cycle action on selected line.
                        (KeyCode::Tab, KeyModifiers::NONE) => {
                            app.apply(Action::CycleRebaseAction);
                        }
                        // Up arrow — Move selection up.
                        (KeyCode::Up, KeyModifiers::NONE) => {
                            app.apply(Action::MoveRebaseUp);
                        }
                        // Down arrow — Move selection down.
                        (KeyCode::Down, KeyModifiers::NONE) => {
                            app.apply(Action::MoveRebaseDown);
                        }
                        // All other keys ignored in rebase mode.
                        _ => {}
                    }
                } else {
                    // Normal editing mode (commit, merge, unknown).
                    match (*code, *modifiers) {
                        // Ctrl+H — Show help overlay.
                        (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                            app.apply(Action::Help);
                        }
                        // Ctrl+S — Save and exit.
                        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                            match app.apply(Action::Save) {
                                Outcome::Save => {
                                    // Restore raw mode BEFORE file write — prevents garbled
                                    // output if the write fails.
                                    drop(guard);
                                    writer::FileWriter::write_atomic(&app.serialized_content(), &path)?;
                                    process::exit(0);
                                }
                                _ => {}
                            }
                        }
                        // Esc — Cancel without saving. Use `(KeyCode::Esc, _)` to accept any
                        // modifier combination, since some terminals send modifiers with Esc.
                        (KeyCode::Esc, _) => {
                            match app.apply(Action::Cancel) {
                                Outcome::Cancel => {
                                    drop(guard);
                                    process::exit(1);
                                }
                                _ => {}
                            }
                        }
                        // Ctrl+U — Delete entire current line (nano-style; NOT undo).
                        // Moves cursor to line head then deletes to end of line.
                        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                            app.textarea_mut().move_cursor(CursorMove::Head);
                            app.textarea_mut().delete_line_by_end();
                        }
                        // Ctrl+Z — Undo last edit.
                        (KeyCode::Char('z'), KeyModifiers::CONTROL) => {
                            app.textarea_mut().undo();
                        }
                        // Ctrl+Y — Redo last undone edit.
                        (KeyCode::Char('y'), KeyModifiers::CONTROL) => {
                            app.textarea_mut().redo();
                        }
                        // Ctrl+W — Delete previous word.
                        (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
                            app.textarea_mut().delete_word();
                        }
                        // Ctrl+D — Delete next word.
                        (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                            app.textarea_mut().delete_next_word();
                        }
                        // Ctrl+C — Copy selection to system clipboard.
                        // Uses textarea internal copy to populate yank buffer, then bridges
                        // to system clipboard via arboard. Silent no-op if clipboard unavailable
                        // (SSH, headless environments).
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            app.textarea_mut().copy();

                            if let Ok(mut clip) = arboard::Clipboard::new() {
                                let _ = clip.set_text(app.textarea().yank_text().to_string());
                            }
                        }
                        // Ctrl+X — Cut selection to system clipboard.
                        (KeyCode::Char('x'), KeyModifiers::CONTROL) => {
                            app.textarea_mut().cut();

                            if let Ok(mut clip) = arboard::Clipboard::new() {
                                let _ = clip.set_text(app.textarea().yank_text().to_string());
                            }
                        }
                        // Ctrl+V — Paste from system clipboard.
                        // If clipboard is unavailable, falls through silently (no paste).
                        (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
                            if let Ok(mut clip) = arboard::Clipboard::new() {
                                if let Ok(text) = clip.get_text() {
                                    app.textarea_mut().insert_str(text);
                                }
                            }
                        }
                        // All other key presses are delegated to TextArea.
                        // This covers: arrow keys, character insertion, backspace,
                        // delete, Home/End, Enter — giving full editing capability.
                        _ => {
                            app.textarea_mut().input(event.clone());
                        }
                    }
                }
            }
            Event::Resize(_, _) => {
                // Terminal resize: just continue the loop — ratatui's
                // terminal.draw() calls autoresize() automatically.
            }
            _ => {}
        }
    }
}
