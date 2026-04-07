---
phase: 01-git-contract-tui-shell
plan: 02
subsystem: terminal
tags: [rust, ratatui, crossterm, raw-mode, panic-hook, drop-guard]

# Dependency graph
requires:
  - phase: 01-01
    provides: Cargo.toml with ratatui/crossterm dependencies + src/context.rs GitContext
provides:
  - TerminalGuard RAII struct that enables raw mode on construction and restores cooked mode on Drop
  - install_panic_hook() that restores cooked mode before printing panic output
  - Manual Terminal construction without alternate screen (IO-06 compliance)
affects: [01-03, all future phases that render UI]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - RAII guard for terminal state (enable on new, disable on drop)
    - Dual-path panic safety (Drop + explicit panic hook)
    - Manual ratatui::Terminal construction (avoids ratatui::init() which enters alternate screen)

key-files:
  created: [src/terminal.rs]
  modified: [src/main.rs]

key-decisions:
  - "Construct ratatui Terminal manually via CrosstermBackend::new() + Terminal::new() — ratatui::init() is a one-liner helper but it calls EnterAlternateScreen, violating IO-06"
  - "Both Drop and panic hook independently call disable_raw_mode() — dual-path ensures restoration even if one path is skipped"
  - "Panic hook captures original hook with take_hook() and chains it — preserves default backtrace/location output after restoring the terminal"

patterns-established:
  - "Terminal init pattern: install_panic_hook() → TerminalGuard::new() — order is mandatory; panic hook must precede any terminal state change"
  - "Error suppression in Drop and panic hook: let _ = disable_raw_mode() — prevents masking the real error with a secondary failure"

requirements-completed: [IO-03, IO-06, PERF-01]

# Metrics
duration: 30min
completed: 2026-03-19
---

# Phase 01 Plan 02: Terminal Safety Foundation Summary

**Drop-based RAII guard + chained panic hook for raw-mode cleanup without alternate screen, using manual ratatui Terminal construction**

## Performance

- **Duration:** ~30 min
- **Started:** 2026-03-19T00:04:18Z (task 1 commit)
- **Completed:** 2026-03-19
- **Tasks:** 2 (1 auto/TDD + 1 checkpoint verified)
- **Files modified:** 2

## Accomplishments

- `src/terminal.rs` with `TerminalGuard` RAII struct — enables raw mode on construction, restores cooked mode on Drop
- `install_panic_hook()` captures original hook, chains it after `disable_raw_mode()` — panic output is readable in a normal terminal
- Zero uses of `EnterAlternateScreen` or `LeaveAlternateScreen` anywhere in the codebase (IO-06 enforced)
- `src/main.rs` wires install_panic_hook() as the absolute first call in main(), before argument parsing
- Manual checkpoint verification: prior shell output visible above editor; no screen wipe; raw mode restored after panic

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement TerminalGuard with Drop cleanup and panic hook** - `19082f0` (feat)

**Plan metadata:** (docs commit — this summary)

## Files Created/Modified

- `src/terminal.rs` — TerminalGuard RAII struct, install_panic_hook(), unit tests
- `src/main.rs` — wires install_panic_hook() as first call; constructs and drops TerminalGuard

## Decisions Made

- Used `CrosstermBackend::new(stdout()) + Terminal::new(backend)` rather than `ratatui::init()` because the init helper calls `EnterAlternateScreen`, which is explicitly prohibited by IO-06. This is the correct manual construction path for main-buffer rendering.
- Both Drop and the panic hook independently call `disable_raw_mode()`. This is intentional dual-path safety — if `TerminalGuard` is in scope and a panic fires, both paths will run; if the guard was moved or the panic fires before the guard is created, the panic hook still restores the terminal.
- `std::panic::take_hook()` is used to capture the original hook before replacing it. This preserves Rust's default backtrace and location output, so developers still see useful panic messages.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

- `cargo test terminal::tests::terminal_guard_drop_restores_raw_mode` runs without a TTY in the test harness, so `TerminalGuard::new()` returns `Err`. This is expected behavior — the test handles both outcomes gracefully. The Drop path was confirmed by the human checkpoint verification step.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Terminal safety foundation is complete and correct for plan 01-03
- `TerminalGuard::terminal()` returns a `&mut ratatui::Terminal<CrosstermBackend<Stdout>>` — plan 01-03 can use this directly for rendering
- No blockers — `install_panic_hook()` + `TerminalGuard` are ready as-designed

---
*Phase: 01-git-contract-tui-shell*
*Completed: 2026-03-19*
