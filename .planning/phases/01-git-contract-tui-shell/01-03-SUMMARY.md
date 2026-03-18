---
phase: 01-git-contract-tui-shell
plan: 03
subsystem: ui
tags: [ratatui, crossterm, tui, event-loop, atomic-write, git-editor]

# Dependency graph
requires:
  - phase: 01-git-contract-tui-shell plan 01
    provides: GitContext enum and detect_context() from src/context.rs
  - phase: 01-git-contract-tui-shell plan 02
    provides: TerminalGuard with Drop cleanup and install_panic_hook() from src/terminal.rs
provides:
  - App struct with Action/Outcome enums and apply() state machine in src/app.rs
  - FileWriter::write_atomic() with temp-then-rename atomicity in src/writer.rs
  - Renderer::render() drawing file content Paragraph + status bar in src/renderer.rs
  - Complete event loop in src/main.rs satisfying the git editor contract
affects: [02-commit-intelligence, 03-editing-core, 04-rebase-squash-mode]

# Tech tracking
tech-stack:
  added: [tempfile (dev-dependency for writer tests)]
  patterns:
    - Outcome enum pattern — App::apply() returns typed Outcome (Save/Cancel/Continue) instead of bool flags
    - Atomic write pattern — write to {path}.tmp then fs::rename; original never opened for writing
    - Drop-before-exit pattern — explicit drop(terminal_guard) before process::exit() ensures raw mode restore
    - Stateless renderer — Renderer struct has no fields; render() takes frame + app references only

key-files:
  created:
    - src/app.rs
    - src/writer.rs
    - src/renderer.rs
  modified:
    - src/main.rs
    - Cargo.toml
    - Cargo.lock

key-decisions:
  - "App::apply() returns Outcome enum not bool — extensible for future editor modes without changing call sites"
  - "write_atomic uses path.with_extension('tmp') — same directory guarantees rename is atomic (same filesystem)"
  - "drop(terminal_guard) before process::exit() — process::exit() bypasses Rust's drop glue; explicit drop is the only safe cleanup path"
  - "File write occurs after raw mode restore — ensures any write error prints cleanly to terminal, not into TUI frame"
  - "Renderer is stateless Phase 1 — no cursor position or scroll state yet; frame area split: content above, 1-line status bar below"

patterns-established:
  - "Outcome enum: App state transitions return typed Outcome variants, not raw booleans"
  - "Atomic file write: always write to .tmp sibling, then rename — never truncate original directly"
  - "Drop-then-exit: always drop(terminal_guard) explicitly before process::exit() in any exit path"
  - "Status bar layout: Ratatui Layout::vertical with content area + 1-row status bar is the UI frame convention"

requirements-completed: [IO-02, IO-04, IO-05]

# Metrics
duration: 35min
completed: 2026-03-19
---

# Phase 01 Plan 03: Event Loop, Renderer, and Atomic Write Summary

**Complete git editor contract: file renders in TUI, Ctrl+S writes atomically and exits 0, Esc exits 1 — verified with live git commit workflow**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-03-19T00:08:00Z
- **Completed:** 2026-03-19T00:45:00Z
- **Tasks:** 3 (2 auto + 1 checkpoint:human-verify)
- **Files modified:** 6

## Accomplishments

- App state machine with Action/Outcome enums satisfies all three key paths (Save, Cancel, Noop/Continue)
- FileWriter::write_atomic() never touches the original file — writes to .tmp sibling then renames atomically
- Renderer splits the frame into a content Paragraph and a 1-line status bar with hotkey hints and git context label
- Complete event loop in main.rs with correct exit codes and terminal restoration before every process::exit() call
- All 17 unit tests pass (5 app::, 3 writer::, 3 terminal::, 6 context::)
- Human verified: file renders, Ctrl+S exits 0, Esc exits 1, git integration works, startup < 100ms, no screen wipe

## Task Commits

Each task was committed atomically:

1. **Task 1: App state, Action/Outcome enums, FileWriter with atomic write** - `24400c0` (feat)
2. **Task 2: Renderer, event loop, and exit code wiring in main** - `afaab95` (feat)
3. **Task 3: Checkpoint human-verify** - approved, no code changes required

**Plan metadata:** (docs commit — recorded below)

## Files Created/Modified

- `src/app.rs` - App struct, Action enum, Outcome enum, apply() state machine; 5 unit tests
- `src/writer.rs` - FileWriter::write_atomic() temp-then-rename implementation; 3 unit tests
- `src/renderer.rs` - Renderer::render() drawing file content + status bar via ratatui Layout
- `src/main.rs` - Full event loop wired to App, Renderer, FileWriter, TerminalGuard, and process::exit()
- `Cargo.toml` - Added tempfile as dev-dependency for writer tests
- `Cargo.lock` - Updated with tempfile dependency tree

## Decisions Made

- **Outcome enum not bool** — App::apply() returns `Outcome::Save | Outcome::Cancel | Outcome::Continue` instead of bool; future editor modes can add new variants without changing all call sites
- **write_atomic uses path.with_extension("tmp")** — placing the temp file in the same directory as the target guarantees fs::rename is atomic (same filesystem inode move); cross-device rename would silently copy-then-delete
- **drop(terminal_guard) before process::exit()** — process::exit() bypasses Rust's drop glue entirely; without the explicit drop call, TerminalGuard's Drop impl (which calls disable_raw_mode) would never run
- **File write after raw mode restore** — any write error message needs to be readable; if terminal was still in raw mode when the error printed, output would be garbled
- **Stateless Renderer for Phase 1** — no cursor position or scroll offset stored; keeps the struct simple until Phase 3 (editing core) needs them

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None — all implementation steps worked as specified. The TDD flow for Task 1 and the integration in Task 2 proceeded without unexpected errors.

## User Setup Required

None — no external service configuration required.

## Requirements Satisfied

This plan, together with plans 01-01 and 01-02, satisfies all 9 Phase 1 requirements:

| Requirement | Description | Implementation |
|-------------|-------------|----------------|
| IO-01 | Binary takes file path as argv[1] | `Cli::parse()` with clap in main.rs |
| IO-02 | Renders file contents in TUI | `Renderer::render()` Paragraph widget |
| IO-03 | No alternate screen (nano-style) | `TerminalGuard::new()` never calls EnterAlternateScreen |
| IO-04 | Ctrl+S saves and exits 0 | Event loop → `Outcome::Save` → `write_atomic()` → `process::exit(0)` |
| IO-05 | Esc cancels and exits 1 | Event loop → `Outcome::Cancel` → `process::exit(1)` |
| IO-06 | Terminal restored on exit and panic | Drop on terminal_guard + install_panic_hook() |
| CTX-01 | Detects git context from filename | `detect_context()` in src/context.rs |
| CTX-02 | Context label shown in status bar | `app.context()` rendered in status bar text |
| PERF-01 | Startup < 100ms | Verified by user (no async runtime, no heavy init) |

## Next Phase Readiness

- The git editor contract is fully complete; `gitmedit` can be set as `git config core.editor gitmedit` today
- Phase 2 (commit intelligence) can build on `App::content()` and `GitContext` to add commit message validation and smart defaults
- `Renderer` is intentionally stateless and minimal; Phase 3 (editing core) will add cursor, scroll, and text mutation
- No blockers for Phase 2

---
*Phase: 01-git-contract-tui-shell*
*Completed: 2026-03-19*
