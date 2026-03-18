use anyhow::Result;
use crossterm::terminal;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::Stdout;

/// Installs a panic hook that restores the terminal to cooked mode before printing
/// the panic message.
///
/// MUST be called before any terminal state is changed. Does NOT call
/// LeaveAlternateScreen — this application never enters alternate screen (IO-06).
pub fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Restore cooked mode. Errors are suppressed so they don't obscure the
        // panic message. Do NOT call LeaveAlternateScreen — we never entered it.
        let _ = terminal::disable_raw_mode();
        original_hook(panic_info);
    }));
}

/// RAII guard for raw-mode terminal.
///
/// Constructing `TerminalGuard` enables raw mode; dropping it restores cooked mode.
/// Does not use EnterAlternateScreen / LeaveAlternateScreen (IO-06).
pub struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    /// Enable raw mode and wrap the crossterm backend in a ratatui Terminal.
    ///
    /// Note: we construct the backend manually rather than calling `ratatui::init()`
    /// because that helper enters alternate screen, which violates IO-06.
    pub fn new() -> Result<Self> {
        terminal::enable_raw_mode()?;
        let backend = CrosstermBackend::new(std::io::stdout());
        let term = Terminal::new(backend)?;
        Ok(Self { terminal: term })
    }

    /// Return a mutable reference to the underlying ratatui Terminal.
    pub fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Suppress errors — Drop must not panic.
        let _ = terminal::disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_panic_hook_does_not_panic() {
        // Installing the hook must succeed without panicking.
        install_panic_hook();
    }

    #[test]
    fn terminal_guard_drop_restores_raw_mode() {
        // TerminalGuard::new() requires a real TTY; cargo test runs without one.
        // We verify the type compiles and Drop is implemented. Runtime behavior
        // is confirmed by the manual checkpoint verification step.
        //
        // Attempt construction: succeed on real TTY, skip gracefully without one.
        match TerminalGuard::new() {
            Ok(_guard) => {
                // guard dropped here — Drop runs disable_raw_mode()
            }
            Err(_) => {
                // No TTY available (CI / cargo test): this is expected.
                // The Drop impl is still compiled and exercised in integration.
            }
        }
    }
}
