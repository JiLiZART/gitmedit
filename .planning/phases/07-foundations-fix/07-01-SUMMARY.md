---
phase: 07-foundations-fix
plan: 01
subsystem: terminal
tags: [crossterm, ratatui, pty, portable-pty, terminal-lifecycle, inline-rendering]

# Dependency graph
requires: []
provides:
  - "TerminalGuard without alternate screen — renders inline into main buffer (IO-06)"
  - "Drop impl with safe error handling via eprintln, no .unwrap() (IO-07)"
  - "PTY integration test in tests/terminal_integration.rs asserting no 1049h/l sequences"
  - "portable-pty dev-dependency for Phase 11 and Phase 12 reuse"
affects: [08-plain-editor-default, 09-nano-chrome, 11-standalone-commit, 12-cross-platform]

# Tech tracking
tech-stack:
  added: ["portable-pty = 0.9.0 (dev-dependency)"]
  patterns:
    - "Inline terminal rendering: CrosstermBackend::new(stdout) without EnterAlternateScreen"
    - "Safe Drop error handling: if let Err(e) = ... { eprintln! } instead of .unwrap()"
    - "Clear-on-exit: Clear(ClearType::All) + MoveTo(0,0) + Show in Drop"
    - "PTY test with kill backstop: incremental read loop + child.kill() to prevent hang on macOS"

key-files:
  created:
    - tests/terminal_integration.rs
  modified:
    - src/terminal.rs
    - Cargo.toml
    - Cargo.lock

key-decisions:
  - "Remove EnterAlternateScreen and EnableMouseCapture entirely (no mouse handlers in event loop)"
  - "Clear(ClearType::All) + MoveTo(0,0) chosen over FromCursorDown for nano-style clean exit"
  - "PTY test uses incremental read + child.kill() backstop to avoid macOS PTY reader hang"
  - "Test marked #[ignore] — runs via cargo test -- --ignored on PTY-capable machines"

patterns-established:
  - "PTY test pattern: spawn binary via CARGO_BIN_EXE_, collect bytes in read-loop thread, kill backstop"
  - "Drop cleanup pattern: let _ = execute!(clear/move); if let Err = disable_raw_mode; if let Err = execute!(Show)"

requirements-completed: [IO-06, IO-07]

# Metrics
duration: 15min
completed: 2026-04-22
---

# Phase 7 Plan 01: Foundations Fix Summary

**TerminalGuard rewritten for inline rendering without alternate screen, with safe Drop cleanup and a portable-pty PTY integration test confirming no EnterAlternateScreen sequences**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-04-22T12:32:56Z
- **Completed:** 2026-04-22T12:48:00Z
- **Tasks:** 2
- **Files modified:** 3 (src/terminal.rs, Cargo.toml, Cargo.lock) + 1 created (tests/terminal_integration.rs)

## Accomplishments

- Removed EnterAlternateScreen, LeaveAlternateScreen, EnableMouseCapture, DisableMouseCapture from terminal.rs — binary no longer enters alternate screen
- Replaced .unwrap() in Drop with eprintln!-on-error pattern; both cleanup lines (disable_raw_mode + cursor Show) use safe error handling
- Added Clear(ClearType::All) + MoveTo(0,0) + Show to Drop for clean shell exit matching nano behavior
- Created tests/terminal_integration.rs: PTY integration test spawns real gitmedit binary, asserts no alternate screen sequences, passes in 1.5s on macOS

## Task Commits

1. **Task 1: Fix TerminalGuard — remove alternate screen, fix Drop safety** - `8555b06` (fix)
2. **Task 2: Add PTY integration test for IO-06 regression guard** - `6ee1c89` (feat)

## Files Created/Modified

- `src/terminal.rs` - Removed alternate screen + mouse capture; safe Drop with Clear + eprintln
- `Cargo.toml` - Added portable-pty = "0.9.0" to [dev-dependencies]
- `Cargo.lock` - Updated for portable-pty
- `tests/terminal_integration.rs` - PTY integration test asserting no EnterAlternateScreen sequences

## Decisions Made

- **Mouse capture removal confirmed:** grep verified 0 mouse event handlers anywhere in the event loop; both EnableMouseCapture and DisableMouseCapture removed cleanly
- **ClearType::All chosen:** preferred over ClearType::FromCursorDown for predictable full-screen clear matching nano behavior; preserves scrollback history (ClearType::Purge was ruled out)
- **PTY test uses kill backstop:** macOS PTY reader hangs on read_to_end even after child exits because the master fd stays open. The test uses an incremental read loop + child.kill() as a timeout backstop to prevent hanging; test passes in 1.5s

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] PTY test reader hang on macOS**
- **Found during:** Task 2 verification (cargo test -- --ignored)
- **Issue:** The plan's provided PTY test used `read_to_end()` which blocks on macOS until the PTY master fd is closed. `child.wait()` before `read_to_end()` did not help because the master fd was still open. Test hung >60s.
- **Fix:** Replaced `read_to_end` with an incremental `read` loop in a background thread. Added `child.kill()` backstop before `child.wait()` to ensure the child exits. Reader thread naturally ends when child exits and PTY slave is closed.
- **Files modified:** tests/terminal_integration.rs
- **Verification:** `cargo test -- --ignored` now exits in 1.52s with 1 test passed
- **Committed in:** 6ee1c89 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - Bug)
**Impact on plan:** Fix was necessary for the test to function on macOS. Pattern is more robust than the original and aligned with research Pitfall 3 guidance on "read timeout" mitigation.

## Issues Encountered

- PTY test reader hang: resolved via kill backstop pattern (see Deviations). Both the original plan test skeleton and updated version produce identical escape sequence assertions — only the read mechanism changed.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Terminal foundation is now trustworthy: inline rendering confirmed, Drop cannot panic
- Phase 8 (plain editor default) can proceed — no terminal lifecycle risks
- Phase 11 (standalone commit) is unblocked: `drop(guard)` before subprocess will now handle cleanup errors gracefully via eprintln instead of panicking
- PTY harness in tests/terminal_integration.rs is ready for Phase 11 E2E and Phase 12 cross-platform reuse

---
*Phase: 07-foundations-fix*
*Completed: 2026-04-22*
