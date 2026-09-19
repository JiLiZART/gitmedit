use std::path::{Path, PathBuf};
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
#[command(
    name = "gitmedit",
    version,
    about = "Fast, distraction-free git editor"
)]
struct Cli {
    /// Path to the file to edit (provided by git). Without it, gitmedit starts a commit.
    path: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    // Must precede any terminal state change.
    terminal::install_panic_hook();

    let Some(path) = Cli::parse().path else {
        commit_without_arguments()
    };

    if !path.exists() {
        eprintln!("error: file not found: {:?}", path);
        process::exit(1);
    }

    let raw = std::fs::read_to_string(&path)?;
    let context = context::detect_context(&path);
    let mut app = App::new(
        &raw,
        context,
        context::git_dir(&path, context),
        context::read_comment_char(),
    );

    app.file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

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
        let Some(action) = keys::map_event(&event, &app) else {
            continue;
        };

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

/// Run `git commit` with this binary as git's editor, and exit with git's code. git writes the
/// message template and reports every failure itself, so nothing is pre-checked here, and the
/// terminal is left alone: the gitmedit instance git launches sets it up.
fn commit_without_arguments() -> ! {
    let editor = match std::env::current_exe() {
        Ok(exe) => shell_quote(&exe),
        Err(e) => {
            eprintln!("gitmedit: could not run git: {e}");
            process::exit(127);
        }
    };
    match process::Command::new("git")
        .arg("commit")
        .env("GIT_EDITOR", editor)
        .status()
    {
        // A child killed by a signal has no code of its own.
        Ok(status) => process::exit(status.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("gitmedit: could not run git: {e}");
            process::exit(127);
        }
    }
}

/// Quote a path as one shell word, since git runs the editor command through a shell.
fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', r"'\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_wraps_and_escapes() {
        assert_eq!(
            shell_quote(Path::new("/usr/bin/gitmedit")),
            "'/usr/bin/gitmedit'"
        );
        assert_eq!(
            shell_quote(Path::new("/opt/my tools/gitmedit")),
            "'/opt/my tools/gitmedit'"
        );
        assert_eq!(shell_quote(Path::new("/a'b")), r"'/a'\''b'");
    }
}
