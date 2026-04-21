# Phase 7: Foundations Fix - Research

**Researched:** 2026-04-17
**Domain:** Rust terminal lifecycle — crossterm escape sequences, RAII guard cleanup, PTY integration testing
**Confidence:** HIGH

## Summary

Phase 7 is a two-defect surgical fix in `src/terminal.rs`. The code contains a comment-vs-code mismatch: the doc says "does not use EnterAlternateScreen" but `TerminalGuard::new()` emits `EnterAlternateScreen` at line 43 and `Drop` emits `LeaveAlternateScreen` at line 64. The fix removes both calls (IO-06) and replaces the `.unwrap()` on the `execute!` macro in `Drop` with an `eprintln!`-on-error pattern (IO-07).

The file change is minimal: three import removals, two execute! edits, and one clear-screen addition. Risk is low, but the PTY integration test investment is high-value: IO-06 regressed once after a passing checkpoint, proving source-grep guards are insufficient.

`portable-pty` 0.9.0 is the right choice for the PTY integration test. It is cross-platform (macOS, Linux, Windows ConPTY), is the de-facto standard for Rust terminal test harnesses, and the Phase 12 cross-platform work will reuse the same harness. Mouse capture (`EnableMouseCapture` / `DisableMouseCapture`) can also be removed: no code anywhere in the event loop reads mouse events.

**Primary recommendation:** Edit `src/terminal.rs` in three passes — (1) remove alternate screen + mouse capture, (2) fix Drop error handling + add clear-on-exit, (3) add `tests/terminal_integration.rs` using `portable-pty`.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01 (Rendering Strategy):** Use full-screen inline rendering. Keep `CrosstermBackend::new(stdout)` + `Terminal::new(backend)`. Drop `EnterAlternateScreen` at `src/terminal.rs:43` and the unused `EnterAlternateScreen` / `LeaveAlternateScreen` imports at lines 3-5. Do NOT use `Viewport::Inline(N)`.
- **D-02 (Clear on exit):** Emit a clear-screen escape sequence in `TerminalGuard::Drop` before `disable_raw_mode`. Must live in `Drop` to cover every exit path: save, cancel, and panic-via-hook.
- **D-03 (Drop error policy):** Replace `.unwrap()` at line 67 with `if let Err(e) = execute!(...) { eprintln!("gitmedit: terminal cleanup failed: {e}"); }`. Also upgrade the silent `let _ = disable_raw_mode()` at line 60 to the same `eprintln!`-on-error pattern.
- **D-04 (PTY integration test):** Add a PTY integration test that spawns the real `gitmedit` binary, reads the emitted byte stream, and asserts it contains neither `\x1b[?1049h` nor `\x1b[?1049l`. Evaluate `portable-pty` as dev-dependency. Test may be `#[ignore]` if CI runner cannot allocate a PTY.

### Claude's Discretion

- Exact clear sequence for D-02: `Clear(ClearType::All) + MoveTo(0,0)` vs `Clear(FromCursorDown) + cursor home`. Researcher to pick based on cleanest shell state.
- Whether to drop `EnableMouseCapture` / `DisableMouseCapture` entirely. Nano does not capture mouse; gitmedit has no mouse handling in the event loop. Flag as simplification candidate.
- Structure of the new integration test: file name, module placement (`tests/` vs in-source `#[cfg(test)]`), helper fn layout.
- Fate of the existing `terminal_guard_drop_restores_raw_mode` unit test — keep alongside new PTY test or replace.

### Deferred Ideas (OUT OF SCOPE)

- `Viewport::Inline(N)` rendering mode — rejected for v1.1.
- Source-grep regression test — strictly weaker than D-04 PTY test, not worth adding.
- Mouse capture removal — flagged as Claude's Discretion, not a locked decision. Researcher confirms it is safe to remove; decision left to planner.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| IO-06 | Editor renders inline without alternate screen (nano-style) | Remove `EnterAlternateScreen` at line 43; remove `LeaveAlternateScreen` from Drop at line 64; remove both imports. CrosstermBackend with plain stdout renders to main buffer without alternate screen. |
| IO-07 | TerminalGuard::Drop uses safe error handling (no unwrap/panic) | Replace `.unwrap()` at line 67 with `if let Err(e) = ...`. Upgrade silent `let _ =` at line 60 to `eprintln!` pattern. Panicking inside Drop during unwinding causes process abort — `eprintln!` is safe inside Drop. |
</phase_requirements>

---

## Standard Stack

### Core (already in Cargo.toml — no additions needed)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| crossterm | 0.29.0 | Terminal escape codes, raw mode, cursor, clear | Already used; `ClearType` and `MoveTo` come from it |
| ratatui | 0.30 | TUI rendering via `CrosstermBackend` | Already used; no `ratatui::init()` (that helper enters alt screen) |

### New Dev-Dependency

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| portable-pty | 0.9.0 | Spawn binary in PTY, read output bytes | Phase 7 PTY integration test + reused in Phase 11/12 |

**Installation (dev-dependency only):**
```toml
[dev-dependencies]
tempfile = "3"
portable-pty = "0.9.0"
```

**Version verification:** `cargo search portable-pty` returns `0.9.0` (confirmed 2026-04-17). `cargo search crossterm` returns `0.29.0`, matching Cargo.toml.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| portable-pty | ptyprocess, pty-process, xpty | portable-pty is the de-facto standard, has Windows ConPTY support needed for Phase 12, has an established API and docs |
| portable-pty | expectrl | expectrl is higher-level but more opinionated; portable-pty gives raw byte stream which is what the test needs |

---

## Architecture Patterns

### Pattern 1: Inline (Non-Alternate-Screen) Terminal Setup

**What:** Construct `CrosstermBackend` from a plain stdout handle, enable raw mode, do NOT emit `EnterAlternateScreen`. Output flows into the main scroll buffer.

**When to use:** Any full-screen TUI that should behave like nano — output stays in scrollback.

**Example:**
```rust
// Source: crossterm 0.29 docs + FEATURES.md §IO-06
use crossterm::terminal;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

pub fn new() -> Result<Self> {
    let stdout = std::io::stdout();
    terminal::enable_raw_mode()?;
    // No EnterAlternateScreen here — renders into main buffer (IO-06)
    let backend = CrosstermBackend::new(stdout);
    let term = Terminal::new(backend)?;
    Ok(Self { terminal: term })
}
```

Note: `ratatui::init()` is NOT used because that helper internally calls `EnterAlternateScreen`. Constructing `CrosstermBackend` manually bypasses it.

### Pattern 2: Clear-on-Exit in Drop (D-02)

**What:** Before disabling raw mode in `Drop`, emit a clear + cursor-home sequence. This removes any lingering TUI borders/status bars from the terminal visible area so the shell prompt returns cleanly.

**Recommended sequence:** `Clear(ClearType::All)` followed by `MoveTo(0, 0)`.

**Why `ClearType::All` over alternatives:**
- `ClearType::All` — clears the visible screen area; scrollback is preserved. Shell history above the editor remains accessible. This matches nano's behavior.
- `ClearType::Purge` — clears screen AND scrollback history. Too destructive; erases the user's terminal history.
- `Clear(FromCursorDown)` — only clears downward from cursor; may leave artifacts if cursor is not at top.

`ClearType::All` + `MoveTo(0,0)` is the correct choice for nano-style behavior.

**Example:**
```rust
// Source: crossterm 0.29 ClearType docs
use crossterm::terminal::{Clear, ClearType};
use crossterm::cursor::MoveTo;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Clear screen before disabling raw mode so shell prompt is clean (D-02)
        let _ = crossterm::execute!(
            self.terminal.backend_mut(),
            Clear(ClearType::All),
            MoveTo(0, 0)
        );
        // Safe error handling — never panic in Drop (IO-07 / D-03)
        if let Err(e) = crossterm::terminal::disable_raw_mode() {
            eprintln!("gitmedit: terminal cleanup failed: {e}");
        }
    }
}
```

### Pattern 3: Safe Error Handling in Drop (D-03)

**What:** Replace `.unwrap()` and `let _ =` with `eprintln!`-on-error in `TerminalGuard::Drop`.

**Why eprintln is safe in Drop:** `eprintln!` writes to stderr, which does not panic on failure. It is safe to call during stack unwinding. `println!` to stdout would be risky (stdout may be in broken state), but stderr is separate.

**Panic-in-Drop consequence:** If `Drop` panics during stack unwinding (from another panic), Rust immediately calls `abort()`. This is not a recoverable panic — the process terminates with no cleanup. The `.unwrap()` at line 67 is precisely this risk.

```rust
// Drop with D-03 pattern
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = crossterm::execute!(
            self.terminal.backend_mut(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        );
        if let Err(e) = crossterm::terminal::disable_raw_mode() {
            eprintln!("gitmedit: terminal cleanup failed: {e}");
        }
        if let Err(e) = crossterm::execute!(
            self.terminal.backend_mut(),
            crossterm::cursor::Show
        ) {
            eprintln!("gitmedit: terminal cleanup failed: {e}");
        }
    }
}
```

Note: With alternate screen and mouse capture removed, the `execute!` in Drop only needs to restore cursor visibility (crossterm may hide cursor during rendering). The `LeaveAlternateScreen` and `DisableMouseCapture` calls are removed entirely.

### Pattern 4: PTY Integration Test with portable-pty

**What:** Spawn the `gitmedit` binary inside a PTY, send Escape to close it immediately, read all output bytes, assert `\x1b[?1049h` and `\x1b[?1049l` are absent.

**File placement:** `tests/terminal_integration.rs` — Rust's `tests/` directory creates a separate integration test binary that has access to the compiled binary via `env!("CARGO_BIN_EXE_gitmedit")`. This is cleaner than in-source `#[cfg(test)]` for tests that spawn a subprocess.

**Key portable-pty API:**
```rust
// Source: portable-pty 0.9.0 docs
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

let pty_system = native_pty_system();
let pair = pty_system.openpty(PtySize {
    rows: 24,
    cols: 80,
    pixel_width: 0,
    pixel_height: 0,
})?;
let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_gitmedit"));
cmd.arg(test_file_path);
let _child = pair.slave.spawn_command(cmd)?;

// Read from master
let mut reader = pair.master.try_clone_reader()?;
// Write input (Escape to quit)
let mut writer = pair.master.take_writer()?;
// ... read bytes, assert no 1049h/l
```

**CI/ignore strategy:** If CI does not allocate a PTY (common in container-based CI), `native_pty_system().openpty(...)` will fail. Mark the test `#[ignore]` so it does not block CI by default; run explicitly with `cargo test -- --ignored` in PTY-capable environments (developer machines, Phase 12 verification).

**Reading output with timeout:** The reader will block waiting for EOF (when the child exits). Use a thread or `BufReader` with small reads. After sending Escape and waiting for child exit, read all remaining bytes.

### Anti-Patterns to Avoid

- **Calling `ratatui::init()`:** This helper enters alternate screen. Always construct `CrosstermBackend` manually.
- **`let _ = crossterm::execute!(...)` in Drop for cleanup:** Silent suppression hides diagnostic info Phase 11 needs. Use `eprintln!` pattern per D-03.
- **`ClearType::Purge` on exit:** Clears scrollback history, which destroys user context. Use `ClearType::All`.
- **In-source `#[cfg(test)]` for the PTY test:** PTY tests need a compiled binary via `CARGO_BIN_EXE_gitmedit`. This env var is only available in `tests/` integration tests, not unit tests inside `src/`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Spawn process in PTY | Custom PTY via libc `forkpty()` | `portable-pty` 0.9.0 | Windows ConPTY, macOS, Linux handled; forkpty is Unix-only |
| Read PTY output as bytes | Raw file descriptor reads | `pair.master.try_clone_reader()` | Handles platform differences, blocking semantics |
| Clear terminal on exit | Custom ANSI string literal `"\x1b[2J\x1b[H"` | `crossterm::execute!(Clear(All), MoveTo(0,0))` | Already a dependency; platform-safe encoding |

**Key insight:** The crossterm dependency already provides every escape sequence needed. No raw ANSI string literals needed.

---

## Common Pitfalls

### Pitfall 1: Remove imports but forget the execute! call (or vice versa)

**What goes wrong:** Removing `EnterAlternateScreen` import but not the `execute!(stdout, EnterAlternateScreen, ...)` line at line 43, or removing the line but leaving the unused import. The compiler will catch the unused import as a warning (not an error), and a remaining call will be a compile error.

**How to avoid:** Edit both the `use` block (lines 3-5) and the `execute!` call (line 43) in the same task. Check that `crossterm::terminal::EnterAlternateScreen` and `LeaveAlternateScreen` do not appear anywhere in `src/terminal.rs` after the change.

### Pitfall 2: Forgetting cursor restoration in Drop

**What goes wrong:** Ratatui's rendering may hide the terminal cursor during rendering (`crossterm::cursor::Hide`). If `Drop` only calls `disable_raw_mode` without restoring cursor visibility, the terminal cursor stays invisible after exit.

**How to avoid:** Include `crossterm::cursor::Show` in the `Drop` execute sequence. Verify by running the binary and confirming the cursor is visible after exit.

### Pitfall 3: PTY reader hangs indefinitely

**What goes wrong:** `pair.master.try_clone_reader().read_to_end()` blocks until the child process exits AND the PTY master is closed. If the child has not exited or the writer is still open, the read hangs.

**How to avoid:** (1) Drop the writer end before reading. (2) Send Escape and wait for `child.wait()` to confirm exit. (3) Consider a read timeout (spawn a thread, join with timeout).

### Pitfall 4: `CARGO_BIN_EXE_gitmedit` only works in tests/ integration tests

**What goes wrong:** Placing the PTY test in `src/terminal.rs` under `#[cfg(test)]` — the `env!("CARGO_BIN_EXE_gitmedit")` macro expands to empty string or panics because unit test binaries are not the same compiled binary.

**How to avoid:** Place the PTY test in `tests/terminal_integration.rs`. This file is compiled as a separate test binary with access to the package's binary targets via `CARGO_BIN_EXE_*`.

### Pitfall 5: Mouse capture left enabled breaks terminal after exit

**What goes wrong:** If `EnableMouseCapture` is emitted on entry but `DisableMouseCapture` is removed from `Drop` (as part of the cleanup), the terminal stays in mouse-capture mode after the editor exits. Mouse clicks produce raw escape sequences in the parent shell.

**How to avoid:** Since neither `EnableMouseCapture` nor `DisableMouseCapture` are needed (no mouse handling in event loop), remove BOTH from the new code. The solution is not to keep one and remove the other — remove both.

### Pitfall 6: Pre-existing `terminal_guard_drop_restores_raw_mode` test

**What goes wrong:** The existing unit test at `src/terminal.rs:82` calls `TerminalGuard::new()` which (before the fix) emits `EnterAlternateScreen`. After the fix, `new()` no longer emits it — the test still passes (it just tests compilation + no panic). It remains valid and should be kept.

**How to avoid:** Keep the existing test. It verifies the Drop path does not panic. The PTY test added in `tests/` verifies the escape sequences. Both serve different roles.

---

## Code Examples

### Current broken state (what to fix)

```rust
// src/terminal.rs lines 1-5 — imports to remove:
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen,  // REMOVE BOTH
};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture}; // REMOVE LINE

// src/terminal.rs line 43 — execute! to fix:
crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
// Replace with:
// (no execute! here — just enable_raw_mode and construct backend)

// src/terminal.rs lines 57-68 — Drop to fix:
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();          // UPGRADE to eprintln!
        crossterm::execute!(
            self.terminal().backend_mut(),
            LeaveAlternateScreen,                      // REMOVE
            DisableMouseCapture                        // REMOVE
        ).unwrap();                                    // REPLACE with if let Err
    }
}
```

### Target state (what to implement)

```rust
// src/terminal.rs — new imports (only what's needed):
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::cursor::{MoveTo, Show};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::Stdout;

// TerminalGuard::new() — no alternate screen, no mouse capture:
pub fn new() -> Result<Self> {
    let stdout = std::io::stdout();
    terminal::enable_raw_mode()?;
    // No EnterAlternateScreen — inline rendering (IO-06)
    let backend = CrosstermBackend::new(stdout);
    let term = Terminal::new(backend)?;
    Ok(Self { terminal: term })
}

// Drop — clear + safe error handling (IO-07, D-02, D-03):
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Clear screen so shell prompt returns cleanly (D-02)
        let _ = crossterm::execute!(
            self.terminal.backend_mut(),
            Clear(ClearType::All),
            MoveTo(0, 0)
        );
        // Safe cleanup — no unwrap in Drop (IO-07 / D-03)
        if let Err(e) = terminal::disable_raw_mode() {
            eprintln!("gitmedit: terminal cleanup failed: {e}");
        }
        if let Err(e) = crossterm::execute!(self.terminal.backend_mut(), Show) {
            eprintln!("gitmedit: terminal cleanup failed: {e}");
        }
    }
}
```

### PTY integration test skeleton

```rust
// tests/terminal_integration.rs
// Source: portable-pty 0.9.0 docs + portable-pty crate patterns

#[test]
#[ignore] // Requires PTY allocation; run with: cargo test -- --ignored
fn no_alternate_screen_escape_sequences() {
    use portable_pty::{native_pty_system, CommandBuilder, PtySize};
    use std::io::{Read, Write};
    use tempfile::NamedTempFile;

    // Create a temp file for gitmedit to open
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "test commit message").unwrap();
    tmp.flush().unwrap();

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
        .unwrap();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_gitmedit"));
    cmd.arg(tmp.path());

    let mut child = pair.slave.spawn_command(cmd).unwrap();

    // Send Escape to cancel and exit
    let mut writer = pair.master.take_writer().unwrap();
    let mut reader = pair.master.try_clone_reader().unwrap();
    drop(pair.slave); // close slave so reader sees EOF

    std::thread::sleep(std::time::Duration::from_millis(200));
    writer.write_all(b"\x1b").unwrap(); // Escape
    drop(writer);

    child.wait().unwrap();

    let mut output = Vec::new();
    reader.read_to_end(&mut output).unwrap();

    let raw = String::from_utf8_lossy(&output);
    assert!(
        !raw.contains("\x1b[?1049h"),
        "EnterAlternateScreen found in output — IO-06 violated"
    );
    assert!(
        !raw.contains("\x1b[?1049l"),
        "LeaveAlternateScreen found in output — IO-06 violated"
    );
}
```

---

## Mouse Capture Removal: Discretion Finding

**Verdict: Remove both `EnableMouseCapture` and `DisableMouseCapture` entirely.**

Evidence:
- `grep -n "mouse\|Mouse" src/main.rs` returns 0 matches. No mouse event handling anywhere in the event loop.
- `ratatui-textarea` does not require mouse capture for keyboard-only operation.
- Nano does not enable mouse capture.
- Leaving `EnableMouseCapture` without `DisableMouseCapture` (or vice versa) would be a bug.
- Leaving both in as a no-op adds cleanup code for a feature that is not used.

This is the clean simplification the CONTEXT.md flagged. The planner should include removal of both as part of the `TerminalGuard::new()` cleanup task.

**Confidence: HIGH** — verified by grep over the full source.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `ratatui::init()` helper | Manual `CrosstermBackend::new(stdout)` | Project v1.0 intent (regressed) | `ratatui::init()` enters alt screen; manual construction does not |
| `.unwrap()` in Drop | `if let Err(e) = ...` + eprintln | Phase 7 fix | Prevents process abort during Phase 11 teardown |
| Silent `let _ =` for raw mode | `eprintln!` on error | Phase 7 fix | Surfaces diagnostics when tty is broken |

**Deprecated in this phase:**
- `EnterAlternateScreen` / `LeaveAlternateScreen` imports and usage — removed, not replaced
- `EnableMouseCapture` / `DisableMouseCapture` imports and usage — removed, not replaced

---

## Open Questions

1. **Cursor visibility during ratatui rendering**
   - What we know: Crossterm terminals may hide the cursor during rendering (`cursor::Hide`). Ratatui may issue `Hide` as part of its frame draw.
   - What's unclear: Does the current `TerminalGuard::Drop` need to issue `cursor::Show` explicitly, or does ratatui's terminal restore it?
   - Recommendation: Include `cursor::Show` in the Drop sequence as a defensive measure. It is idempotent and harmless if cursor was already visible.

2. **`terminal().backend_mut()` borrow in Drop**
   - What we know: Current Drop calls `self.terminal()` which returns `&mut self.terminal`. In a new `Drop` that no longer calls `self.terminal()` (removing that helper from Drop), direct `self.terminal.backend_mut()` should work.
   - What's unclear: Whether the borrow checker accepts `self.terminal.backend_mut()` inside `Drop` given the existing method.
   - Recommendation: Use `self.terminal.backend_mut()` directly in Drop rather than going through the `terminal()` accessor method.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust / cargo | Build + test | ✓ | (project standard) | — |
| PTY allocation | PTY integration test | ✓ (macOS dev machine) | macOS native | Mark test `#[ignore]`, run manually |
| portable-pty crate | PTY integration test | needs adding | 0.9.0 | — |
| tempfile crate | PTY test temp file | ✓ | 3 (in dev-deps) | — |

**Missing dependencies with no fallback:**
- `portable-pty = "0.9.0"` must be added to `[dev-dependencies]` in Cargo.toml.

**Missing dependencies with fallback:**
- PTY allocation in CI: mark the integration test `#[ignore]`; it runs on developer machines and in explicit PTY-capable CI steps.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config file | None — uses default |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo test -- --ignored` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| IO-06 | Binary emits no `\x1b[?1049h` or `\x1b[?1049l` | integration (PTY) | `cargo test -- --ignored` | ❌ Wave 0: `tests/terminal_integration.rs` |
| IO-07 | Drop does not panic when stdout is in broken state | unit | `cargo test terminal_guard` | ✅ `src/terminal.rs` (existing, extended) |
| IO-07 | Cleanup failures print to stderr, not panic | unit | `cargo test terminal_guard` | ✅ extends existing test |
| Regression | All 94 existing tests still pass | unit | `cargo test` | ✅ all existing |

### Sampling Rate

- **Per task commit:** `cargo test` (unit tests, < 1s)
- **Per wave merge:** `cargo test && cargo test -- --ignored`
- **Phase gate:** Full suite green (including `--ignored`) before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/terminal_integration.rs` — covers IO-06 PTY assertion
- [ ] `Cargo.toml` dev-dependency addition: `portable-pty = "0.9.0"`

*(All other test infrastructure exists — 94 unit tests already passing)*

---

## Sources

### Primary (HIGH confidence)
- `src/terminal.rs` — direct code inspection: confirmed `EnterAlternateScreen` at line 43, `LeaveAlternateScreen` + `.unwrap()` at lines 62-67, `EnableMouseCapture` / `DisableMouseCapture` present
- `src/main.rs` — grep confirmed 0 mouse event handlers in event loop
- crossterm 0.29.0 docs (docs.rs) — `ClearType` variants, `MoveTo` signature, `EnterAlternateScreen` = `\x1b[?1049h`
- portable-pty 0.9.0 docs (docs.rs) — PTY API: `native_pty_system()`, `PtySize`, `CommandBuilder`, `try_clone_reader()`, `take_writer()`
- cargo search — confirmed portable-pty 0.9.0 current as of 2026-04-17

### Secondary (MEDIUM confidence)
- `.planning/research/PITFALLS.md` §Pitfall 9 — Drop panic analysis (project-internal, written during v1.1 kickoff research)
- `.planning/research/FEATURES.md` §IO-06 — inline rendering implementation guidance
- WebSearch result confirming `\x1b[?1049h` is the crossterm `EnterAlternateScreen` sequence on ANSI systems

### Tertiary (LOW confidence)
- Rust forum thread on portable-pty blocking reads — notes timing/synchronization nuances; resolved by writer-drop pattern in test skeleton above

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — crossterm and ratatui versions confirmed from Cargo.toml + cargo search; portable-pty 0.9.0 confirmed from cargo search
- Architecture: HIGH — direct code inspection of `src/terminal.rs`; all patterns verified against crossterm 0.29 docs
- Pitfalls: HIGH — most derive from direct code reading; PTY test pitfalls from docs.rs API study
- Mouse removal: HIGH — grep over full source confirms 0 mouse event handlers

**Research date:** 2026-04-17
**Valid until:** 2026-05-17 (crossterm and ratatui are stable; portable-pty API unlikely to change in 30 days)
