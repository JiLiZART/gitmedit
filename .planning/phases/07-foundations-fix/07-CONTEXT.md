# Phase 7: Foundations Fix - Context

**Gathered:** 2026-04-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Fix two surgical defects in `src/terminal.rs` so the editor has a trustworthy terminal foundation before any other v1.1 work:

1. **IO-06** — Remove `EnterAlternateScreen` / `LeaveAlternateScreen`. gitmedit must render inline into the main terminal buffer (nano model), not into an alternate screen.
2. **IO-07** — `TerminalGuard::Drop` must never panic. Current `.unwrap()` on cleanup `execute!` aborts the process if drop runs while stdout is in an unexpected state — a time bomb for Phase 11 (standalone commit) where `drop(guard)` runs before `git commit` subprocess.

**Scope anchor:** One file edited (`src/terminal.rs`), one new integration test file added, all 94 existing tests pass, no behavior changes outside terminal lifecycle.

**Out of scope (other phases):** plain editor default (Phase 8), nano chrome bars (Phase 9), rebase reorder (Phase 10), standalone commit (Phase 11), cross-platform verification (Phase 12).

</domain>

<decisions>
## Implementation Decisions

### Rendering Strategy

- **D-01:** Use **full-screen inline rendering**. Keep `CrosstermBackend::new(stdout)` + `Terminal::new(backend)` pattern. Drop the `EnterAlternateScreen` call at `src/terminal.rs:43` and the unused `EnterAlternateScreen` / `LeaveAlternateScreen` imports at `src/terminal.rs:3-5`. Do NOT use `Viewport::Inline(N)` — that mode is for small embedded widgets (progress bars), not full editors. Nano itself uses full-screen non-alternate-screen rendering.

### Terminal Cleanup on Exit

- **D-02:** Clear the terminal on exit. In `TerminalGuard::Drop`, before `disable_raw_mode`, emit a clear-screen escape sequence so the shell prompt returns cleanly after the editor exits (no stuck borders/status bars in scrollback). This must live in `Drop` so it covers every exit path: save, cancel, and panic-via-hook. Matches nano's behavior.

### Drop Error Policy

- **D-03:** **Stderr warn on cleanup failure — both lines.** Replace the `.unwrap()` at `src/terminal.rs:67` with `if let Err(e) = execute!(...) { eprintln!("gitmedit: terminal cleanup failed: {e}"); }`. Also upgrade the currently-silent `let _ = disable_raw_mode()` at `src/terminal.rs:60` to the same eprintln-on-error pattern. Rationale: Phase 11's standalone commit mode runs `drop(guard)` before spawning `git commit`; if cleanup fails, git runs in a broken tty and the user needs a diagnostic. `eprintln!` is safe inside `Drop` (does not panic on success).

### Regression Test

- **D-04:** Add a **PTY integration test** that spawns the real `gitmedit` binary in a pseudo-terminal, reads the emitted byte stream, and asserts it contains neither `\x1b[?1049h` (EnterAlternateScreen) nor `\x1b[?1049l` (LeaveAlternateScreen). Researcher to evaluate `portable-pty` as dev-dependency (cross-platform, works on Windows for Phase 12). Deliberate investment in a reusable PTY harness — Phase 11 standalone commit E2E and Phase 12 cross-platform verification will reuse it. Test may be marked `#[ignore]` and run via explicit `cargo test --ignored` if CI runner cannot allocate a PTY.

### Claude's Discretion

- Exact clear sequence for D-02: `Clear(ClearType::All) + MoveTo(0,0)` vs `Clear(FromCursorDown) + cursor home`. Researcher should pick based on which leaves the cleanest shell state on all supported terminals.
- Whether to drop `EnableMouseCapture` / `DisableMouseCapture` entirely. Nano does not capture mouse; gitmedit has no mouse handling in the event loop. Flag as a simplification candidate — do not change behavior silently.
- Structure of the new integration test: file name, module placement (`tests/` directory vs `#[cfg(test)]` in-source), helper fn layout.
- Fate of the existing `terminal_guard_drop_restores_raw_mode` unit test — keep alongside new PTY test or replace.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 7 Requirements & Audit

- `.planning/REQUIREMENTS.md` — IO-06 and IO-07 acceptance criteria
- `.planning/ROADMAP.md` §"Phase 7: Foundations Fix" — success criteria (3 items)
- `.planning/milestones/v1.0-MILESTONE-AUDIT.md` §53, §83, §171, §196 — original IO-06 regression evidence and TerminalGuard Drop panic analysis

### Project-Level Context

- `.planning/PROJECT.md` §"Key Decisions" — "No alternate screen (IO-06)" and "Drop-based terminal cleanup" decisions
- `.planning/STATE.md` §"Decisions" — "TerminalGuard Drop uses .unwrap() — known time bomb; must fix before standalone commit (Phase 7, before Phase 11)"

### Prior Research (from v1.1 kickoff)

- `.planning/research/ARCHITECTURE.md` §"terminal.rs" — notes the comment-vs-code inconsistency in IO-06
- `.planning/research/PITFALLS.md` §"TerminalGuard::Drop .unwrap()" (lines ~271–282) — full analysis of the Drop panic risk, including interaction with standalone commit mode
- `.planning/research/FEATURES.md` §"IO-06" (lines ~245–252) — implementation guidance for non-alt-screen rendering with ratatui + CrosstermBackend
- `.planning/research/SUMMARY.md` §"Rationale" (line ~82) — why Phase 7 must land first

### Code to Modify

- `src/terminal.rs` — the only source file changed by this phase
- `src/main.rs:49` — `TerminalGuard::new()` construction site (read-only for this phase; no changes expected)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`TerminalGuard` struct** (`src/terminal.rs:30`): RAII wrapper already in place. Phase 7 modifies it in-place — no new types.
- **`install_panic_hook()`** (`src/terminal.rs:16`): Already correct — comment says "Does NOT call LeaveAlternateScreen" and code matches. No change needed here.
- **`drop(guard)` before `process::exit()` pattern** (`src/main.rs:84, 95, 129, 141`): Already used on every save/cancel path. This is the contract Phase 11 depends on — Phase 7 must preserve it.

### Established Patterns

- **Error handling in `main.rs`**: Uses `anyhow::Result` + `?` operator. `Drop` impls use silent error suppression (`let _ =`). D-03 upgrades the silent pattern to `eprintln!` specifically for `TerminalGuard::Drop` — does not change the global pattern.
- **Test layout**: Existing unit tests live in `#[cfg(test)] mod tests` inside each source file. No `tests/` directory exists yet. The PTY integration test in D-04 will likely create it (researcher to confirm).
- **Dev-dependencies**: Project currently has none beyond what `ratatui` / `crossterm` / `arboard` pull in. `portable-pty` would be the first dedicated dev-dep — justify in research.

### Integration Points

- `src/main.rs:49` — only construction site for `TerminalGuard`. No other code touches terminal lifecycle.
- `src/main.rs:84, 95, 129, 141` — explicit `drop(guard)` before `process::exit()` on save/cancel. D-02's clear-on-exit runs inside `Drop`, so these sites need no changes.
- Panic hook installed at `src/main.rs:27` via `terminal::install_panic_hook()` — already correct, flagged read-only for this phase.

</code_context>

<specifics>
## Specific Ideas

- "Like nano" means: full-screen rendering, no alt screen, clean terminal on exit, shell prompt returns on a fresh line with previous scrollback preserved above.
- IO-06 already regressed once (after Phase 1 VERIFICATION passed, per `v1.0-MILESTONE-AUDIT.md:53`). That is the direct motivation for investing in a PTY integration test rather than a cheaper source-grep — manual and low-effort guards already failed on this exact defect.
- Phase 11 (standalone commit) is the downstream consumer that will detonate IO-07 if left unfixed. Phase 7 must land before Phase 11 planning starts.

</specifics>

<deferred>
## Deferred Ideas

- **`Viewport::Inline(N)` rendering mode** — rejected for v1.1. Could revisit in v1.2+ if a more terminal-native "small embedded editor" mode is ever desired. Not a priority.
- **Source-grep regression test** — strictly weaker than the PTY integration test chosen in D-04. Not worth adding alongside.
- **Mouse capture removal** — flagged as Claude's Discretion in D-01, not a locked decision. If researcher confirms nothing in the event loop reads mouse events, it becomes a clean simplification; otherwise leave it alone.

### Reviewed Todos (Not Folded)

- `2026-03-23-decide-v1-milestone-scope-beta-vs-complete-delivery.md` — archived v1.0 planning todo, not relevant to Phase 7.
- `2026-03-24-add-git-commit-mode-for-quick-commit-without-file.md` — already mapped to Phase 11 (standalone commit mode), not Phase 7.

</deferred>

---

*Phase: 07-foundations-fix*
*Context gathered: 2026-04-15*
</content>
</invoke>