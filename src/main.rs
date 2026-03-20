use std::path::PathBuf;
use std::process;

use clap::Parser;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

mod app;
mod context;
mod document;
mod renderer;
mod terminal;
mod writer;

use app::{Action, App, Outcome};

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
            Event::Key(KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            }) => match app.apply(Action::Save) {
                Outcome::Save => {
                    // Restore raw mode BEFORE file write — prevents garbled
                    // output if the write fails.
                    drop(guard);
                    writer::FileWriter::write_atomic(&app.serialized_content(), &path)?;
                    process::exit(0);
                }
                _ => {}
            },
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            }) => match app.apply(Action::Cancel) {
                Outcome::Cancel => {
                    drop(guard);
                    process::exit(1);
                }
                _ => {}
            },
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                ..
            }) => {
                // All other key presses are delegated to TextArea.
                // This covers: arrow keys, character insertion, backspace,
                // delete, Home/End, Enter — giving full editing capability.
                app.textarea_mut().input(event.clone());
            }
            Event::Resize(_, _) => {
                // Terminal resize: just continue the loop — ratatui's
                // terminal.draw() calls autoresize() automatically.
            }
            _ => {}
        }
    }
}
