use anyhow::Result;
use crossterm::cursor::Show;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{Stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};

/// Whether `TerminalGuard` has set the terminal up and it has not been restored yet.
static ACTIVE: AtomicBool = AtomicBool::new(false);

/// Write the sequences that undo `TerminalGuard::new`: mouse capture off, alternate screen left,
/// cursor shown. Every step runs even if an earlier one fails, and errors are discarded, because
/// this runs from `Drop` and from the panic hook, where a second panic aborts the process.
pub fn write_restore_sequences(out: &mut impl Write) {
    let _ = crossterm::execute!(out, DisableMouseCapture);
    let _ = crossterm::execute!(out, LeaveAlternateScreen);
    let _ = crossterm::execute!(out, Show);
}

/// Fully restore the user's terminal, once per setup. The panic hook restores before the panic
/// message is printed; the guard's `Drop` then runs while unwinding and must not emit the sequences
/// again, because leaving the alternate screen a second time moves the cursor back over the message.
pub fn restore_terminal() {
    if restore_if_active(&mut std::io::stdout()) {
        let _ = terminal::disable_raw_mode();
    }
}

fn restore_if_active(out: &mut impl Write) -> bool {
    let active = ACTIVE.swap(false, Ordering::SeqCst);
    if active {
        write_restore_sequences(out);
    }
    active
}

/// Installs a panic hook that restores the terminal before the panic message is printed.
/// MUST be called before any terminal state is changed.
pub fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        restore_terminal();
        original_hook(panic_info);
    }));
}

/// RAII guard: raw mode, alternate screen, and mouse capture while alive; fully restored on drop.
pub struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    pub fn new() -> Result<Self> {
        let mut stdout = std::io::stdout();
        terminal::enable_raw_mode()?;
        ACTIVE.store(true, Ordering::SeqCst);
        if let Err(e) = crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
            restore_terminal();
            return Err(e.into());
        }
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok(Self { terminal })
    }

    pub fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = self.terminal.backend_mut().flush();
        restore_terminal();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[cfg(unix)]
    #[test]
    fn restore_sequences_disable_mouse_then_leave_alt_screen_then_show_cursor() {
        let mut buf = Vec::new();
        write_restore_sequences(&mut buf);
        let s = String::from_utf8(buf).unwrap();
        let mouse = s.find("\x1b[?1000l").expect("mouse capture disabled");
        let alt = s.find("\x1b[?1049l").expect("alternate screen left");
        let cursor = s.find("\x1b[?25h").expect("cursor shown");
        assert!(mouse < alt && alt < cursor, "wrong order: {s:?}");
    }

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("closed"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("closed"))
        }
    }

    #[test]
    fn restore_runs_only_once_per_setup() {
        ACTIVE.store(true, Ordering::SeqCst);
        let mut first = Vec::new();
        assert!(restore_if_active(&mut first));
        assert!(!first.is_empty());
        let mut second = Vec::new();
        assert!(!restore_if_active(&mut second));
        assert!(second.is_empty());
    }

    #[test]
    fn restore_sequences_survive_write_errors() {
        write_restore_sequences(&mut FailingWriter);
    }

    #[test]
    fn install_panic_hook_does_not_panic() {
        install_panic_hook();
    }
}
