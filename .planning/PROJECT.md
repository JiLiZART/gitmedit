# gitmedit

## What This Is

A lightweight TUI git editor that replaces heavier defaults (vim, nano) as the global git editor. Handles commits, merges, rebases, and squashes with a minimal interface, context-aware UI, and fast startup. Set it once via `git config --global core.editor gitmedit`.

## Core Value

Provide a fast, distraction-free git editor that feels like nano's simplicity but understands git's context (commits, merges, rebases, squashes) without bloat.

## Current State

Shipped v1.0 MVP with 2,448 LOC Rust across 6 phases (14 plans).
v1.1 Phase 7 complete — inline terminal rendering restored, TerminalGuard Drop panic-free.
Tech stack: ratatui 0.30, crossterm 0.29, clap 4.6, arboard (clipboard).
Binary starts in 17ms, compiles to 663KB release.

## Requirements

### Validated

**Phase 01 (git-contract-tui-shell):**
- ✓ IO-01–IO-05: File read, TUI display, save/cancel exit codes — v1.0
- ✓ CTX-01, CTX-02: Git context detection (commit/merge/rebase/squash/tag) — v1.0
- ✓ PERF-01: Startup < 100ms (17ms achieved) — v1.0

**Phase 02 (text-editing-comment-handling):**
- ✓ EDIT-01, EDIT-02, EDIT-04, EDIT-05, EDIT-06: Full text editing, multiline, comment styling — v1.0
- ✓ CTX-03–CTX-06: Comment char detection, parsing, protection, preservation — v1.0
- ✓ COMMIT-04, COMMIT-05: Commit editing features — v1.0
- ✓ MERGE-01–MERGE-03: Merge conflict styling — v1.0

**Phase 03 (commit-message-intelligence):**
- ✓ COMMIT-01–COMMIT-03, COMMIT-06: Subject counter, blank line, character limits — v1.0
- ✓ HELP-01–HELP-04: Context-aware help overlay — v1.0

**Phase 04 (rebase-squash-modes):**
- ✓ REBASE-01–REBASE-06: Structured table, action cycling, save format — v1.0
- ✓ SQUASH-01–SQUASH-04: Squash log display, editable message, protection — v1.0

**Phase 05 (installation-distribution):**
- ✓ INSTALL-01–INSTALL-04: Cargo install, PATH, git config — v1.0

**Phase 06 (rebase-view-horizontal-scrolling):**
- ✓ REBASE-02 (enhanced): Word-wrapped subjects in rebase table — v1.0

**Phase 07 (foundations-fix):**
- ✓ IO-06: Inline rendering (no alternate screen) — v1.1
- ✓ IO-07: Panic-free TerminalGuard Drop — v1.1

### Known Gaps (from v1.0 audit)

- EDIT-03: Ctrl+U delete line — pending formal verification
- EDIT-07: Undo/redo — pending formal verification
- PERF-02: 30+ FPS rendering — pending formal verification
- PERF-03: No lag on large files — pending formal verification
- INSTALL-05: Editor respects both editor configs — pending

### Active (v1.1)

- [ ] Plain editor default — remove comment protection/read-only logic, use ratatui_textarea default
- [ ] Nano-style chrome — top header with folder name, filename in toolbar, bottom command bar (^S/Esc)
- [ ] Standalone commit mode — `gitmedit` with no args in git repo commits via `git commit -F`
- [ ] Merge commit toolbar — parse comment block for conflict/affected files, display in status bar
- [ ] Rebase line reordering (move commits up/down in todo)
- [ ] Exec line argument editing in rebase mode
- [ ] Cross-platform testing (Windows terminal, iTerm2)
- [ ] Custom hotkey configuration

## Current Milestone: v1.1 Standalone Commit + Editor Overhaul

**Goal:** Add standalone commit mode, simplify editor to nano-like behavior with nano-style chrome, add merge toolbar intelligence, and close v1.0 gaps.

**Target features:**
- Plain editor default (remove comment protection, all lines editable)
- Nano-style chrome (header bar, filename, command bar) across all modes
- Standalone commit mode (no-args git commit via temp file)
- Merge commit toolbar (conflict/affected file info)
- Fix IO-06 alternate screen regression
- Rebase line reordering
- Exec line argument editing
- Cross-platform testing
- Custom hotkey configuration

### Out of Scope

- Vim/Emacs keybindings — standard shortcuts only for simplicity
- Complex syntax highlighting — keep UI minimal
- Plugin system — single focused tool, not extensible
- Graphical UI — terminal only
- Configuration file complexity — sensible defaults

## Context

- **Codebase:** 2,448 LOC Rust, single-crate structure
- **Dependencies:** ratatui 0.30, crossterm 0.29, clap 4.6, arboard, ratatui-textarea 0.8
- **Git context awareness:** Handles commit, merge, rebase, squash, tag contexts
- **Tech debt:** Dead code in Document (comment-protection paths — Phase 8 removes)

## Constraints

- **Language**: Rust
- **UI Library**: ratatui + crossterm
- **Platform**: Linux, macOS (stretch: Windows)
- **Performance**: Must start faster than vim/nano alternatives
- **Git integration**: Must respect git's file formats exactly (no corruption risk)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Single crate (no workspace) | Keep complexity low for v1 | ✓ Good |
| No alternate screen (IO-06) | Preserve terminal output like nano | ✓ Restored in Phase 7 |
| ratatui 0.30 + crossterm 0.29 | Modern Rust TUI with cross-platform | ✓ Good |
| Standard hotkeys (Ctrl+S/Esc) | Users expect standard shortcuts | ✓ Good |
| Word-wrap over horizontal scroll | Simpler UX, no state tracking | ✓ Good |
| Drop-based terminal cleanup | RAII ensures cleanup even on panic | ✓ Good |
| Atomic file writes (rename) | Prevents data corruption | ✓ Good |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-22 after Phase 7 (foundations-fix) completion*
