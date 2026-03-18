use std::path::PathBuf;

use clap::Parser;

mod context;
mod terminal;

#[derive(Parser, Debug)]
#[command(name = "gitmedit", about = "Fast, distraction-free git editor")]
struct Cli {
    /// Path to the file to edit (provided by git)
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    terminal::install_panic_hook();

    let cli = Cli::parse();
    let _ctx = context::detect_context(&cli.path);

    // Acquire raw-mode terminal guard — dropped immediately (zero-length stub session).
    // This exercises the cleanup path before rendering is added in later plans.
    let _guard = terminal::TerminalGuard::new()?;

    Ok(())
}
