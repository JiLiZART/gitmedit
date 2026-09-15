use std::path::PathBuf;
use std::process;
use std::time::Duration;

use clap::Parser;
use crossterm::event;

use session::{App, Outcome};

mod context;
mod details;
mod keys;
mod layout;
mod message;
mod rebase;
mod reword;
mod session;
mod status;
mod terminal;
mod ui;
mod wrap;
mod writer;

#[derive(Parser, Debug)]
#[command(name = "gitmedit", version, about = "Fast, distraction-free git editor")]
struct Cli {
    /// Path to the file to edit (provided by git)
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    // Must precede any terminal state change.
    terminal::install_panic_hook();

    let path = Cli::parse().path;
    if !path.exists() {
        eprintln!("error: file not found: {:?}", path);
        process::exit(1);
    }
    let raw = std::fs::read_to_string(&path)?;
    let context = context::detect_context(&path);
    let mut app = App::new(&raw, context, context::git_dir(&path, context), context::read_comment_char());
    app.file_name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();

    let mut guard = terminal::TerminalGuard::new()?;
    loop {
        guard.terminal().draw(|frame| ui::render(frame, &mut app))?;

        // Wake up regularly so commit details loaded in the background get drawn.
        if !event::poll(Duration::from_millis(100))? {
            app.poll_background();
            continue;
        }
        let event = event::read()?;
        app.poll_background();
        let Some(action) = keys::map_event(&event, &app) else { continue };

        match app.apply(action) {
            Outcome::Continue => {}
            Outcome::Cancel => {
                drop(guard);
                process::exit(1);
            }
            Outcome::Save => {
                let content = app.serialized_content();
                // Restore the terminal before writing so any error message is readable.
                drop(guard);
                writer::FileWriter::write_atomic(&content, &path)?;
                app.finish_save()?;
                process::exit(0);
            }
        }
    }
}
