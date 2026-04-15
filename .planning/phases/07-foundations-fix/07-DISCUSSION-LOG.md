# Phase 7: Foundations Fix - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-15
**Phase:** 07-foundations-fix
**Areas discussed:** Inline rendering approach, Exit visual behavior, Drop error policy, Regression test strategy

---

## Inline Rendering Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Full-screen inline | Drop `EnterAlternateScreen`, keep `CrosstermBackend::new(stdout)` + `Terminal::new(backend)`. TUI takes full terminal height, renders inline into main buffer. Minimal diff. | ✓ |
| `Viewport::Inline(N)` | Use `Terminal::with_options(TerminalOptions { viewport: Viewport::Inline(h) })`. Reserves N lines below cursor. Intended for small embedded widgets, not editors. | |

**User's choice:** Full-screen inline
**Notes:** Matches nano's model exactly. Nano uses full-screen non-alt-screen rendering — takes whole terminal, clears on exit. `Viewport::Inline` is for progress bars and small overlays, not full editors. Minimal code change from current state (just drop the `EnterAlternateScreen` call and unused imports).

---

## Exit Visual Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Clear screen on exit | `execute!` clear in `Drop` before `disable_raw_mode`. Clean terminal, matches nano. | ✓ |
| Leave final frame | Do nothing extra. Last rendered TUI frame stays in scrollback. Borders and status bars stuck there. | |
| Clear drawn region only | `terminal.clear()` before drop. Clears ratatui's buffer area. | |

**User's choice:** Clear screen on exit in `Drop`
**Notes:** Must run inside `Drop` so it covers all `process::exit()` paths in `main.rs` (save, cancel, panic-via-hook). Exact clear variant left as Claude's Discretion — researcher to evaluate `Clear(ClearType::All)` + `MoveTo(0,0)` vs `Clear(FromCursorDown)` + cursor home.

---

## Drop Error Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Silent | `let _ = crossterm::execute!(...)`. Simplest. Matches `disable_raw_mode` line above it. | |
| Stderr warn | `if let Err(e) = execute!(...) { eprintln!("gitmedit: terminal cleanup failed: {e}"); }`. Surfaces diagnostic. | ✓ |
| Log both | Same eprintln treatment for `disable_raw_mode` too — upgrade the existing silent `let _ =` at line 60. | ✓ |

**User's choice:** Stderr warn — both lines
**Notes:** Motivated by Phase 11 (standalone commit), where `drop(guard)` runs *before* `git commit` subprocess. If cleanup fails, git runs in a broken tty and the user must be able to diagnose why. `eprintln!` does not panic on success, so it's safe inside `Drop`. Upgrades both the cleanup `execute!` (line 67) and the existing silent `disable_raw_mode` (line 60).

---

## Regression Test Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Source-grep test | Read `terminal.rs` at test time, assert no `EnterAlternateScreen` string. Crude but cheap. | |
| Manual checkpoint only | Document in plan, verify by eye. What v1.0 did. Failed — IO-06 regressed. | |
| PTY integration test | Spawn gitmedit in a PTY, assert no alt-screen escape sequences emitted. Heavier dep. | ✓ |
| Clippy/grep lint in CI | Shell step `! grep -q EnterAlternateScreen src/terminal.rs`. | |

**User's choice:** PTY integration test
**Notes:** Assert output contains neither `\x1b[?1049h` nor `\x1b[?1049l`. Researcher to evaluate `portable-pty` as dev-dependency — cross-platform, reusable for Phase 11 (standalone commit E2E) and Phase 12 (Windows verification). May be marked `#[ignore]` and run via explicit `cargo test --ignored` if CI runner lacks PTY allocation. Deliberate investment in reusable test harness, not just an IO-06 guard.

---

## Claude's Discretion

- Exact `Clear` variant for D-02 (`ClearType::All` vs `FromCursorDown`).
- Whether to drop `EnableMouseCapture` / `DisableMouseCapture` — nano does not capture mouse; evaluate during research.
- Structure of the new integration test file (name, module layout, helper fns).
- Fate of the existing `terminal_guard_drop_restores_raw_mode` unit test — keep alongside PTY test or replace.

## Deferred Ideas

- `Viewport::Inline(N)` rendering — rejected for this phase, could revisit in v1.2+.
- Source-grep lint in CI — strictly weaker than PTY test, not worth adding.
- Mouse capture removal — flagged as discretion, not locked.

## Reviewed Todos (Not Folded)

- `2026-03-23-decide-v1-milestone-scope-beta-vs-complete-delivery.md` — archived v1.0 planning todo, out of scope.
- `2026-03-24-add-git-commit-mode-for-quick-commit-without-file.md` — already mapped to Phase 11.
</content>
</invoke>