# gitmedit

## What This Is

A lightweight TUI git editor that replaces heavier defaults (vim, nano) as the global git editor. Set it once via `git config --global core.editor gitmedit` and it handles commits, merges, rebases, and squashes with a minimal interface and fast startup.

## Core Value

Provide a fast, distraction-free git editor that feels like nano's simplicity but understands git's context (merges, rebases, squashes) without bloat.

## Requirements

### Validated

**Phase 01 (git-contract-tui-shell):**
- ✓ User can set gitmedit as global git editor
- ✓ User sees TUI window with editable message area
- ✓ User can commit with Ctrl+S save (exit 0)
- ✓ User can cancel with Esc (exit 1)
- ✓ Editor handles commit message files (COMMIT_EDITMSG)
- ✓ Exit status codes match git expectations
- ✓ Editor detects git context (commit, merge, rebase, squash, tag)
- ✓ Startup < 100ms (17ms on developer hardware)
- ✓ Terminal output preserved (no screen wipe)

**Phase 03 (commit-message-intelligence):**
- ✓ Subject line shows real-time character counter (green ≤50, yellow 51-72, red >72)
- ✓ Blank line enforced/suggested between subject and body
- ✓ Hotkey help overlay visible on Ctrl+H (context-aware)
- ✓ Help overlay does not interfere with editing

**Phase 04 (rebase-squash-modes):**
- ✓ Interactive rebase opens structured table (pick, squash, fixup, drop, exec)
- ✓ User cycles action types with Tab (pick→squash→fixup→drop→pick)
- ✓ Comment lines preserved and protected (not editable or cycled)
- ✓ Rebase-todo saved in exact git format (no corruption)
- ✓ Squash mode detects SQUASH_MSG file
- ✓ Squash mode displays commit log as read-only header
- ✓ Squash mode allows editing combined message below log
- ✓ Commit log protected (visually distinct, not selectable)

### Active

- [ ] Full text editing (Ctrl+U delete line, Undo/redo)
- [ ] Rebase line reordering (move commits up/down in todo)
- [ ] Exec line argument editing in rebase mode
- [ ] User can install with `cargo install` and set as global editor
- [ ] Cross-platform (Windows terminal, iTerm2, etc.)
- [ ] Custom hotkey configuration (not hardcoded to Ctrl+S/Esc)

### Out of Scope

- Vim/Emacs keybindings — standard shortcuts only for simplicity
- Complex syntax highlighting — keep UI minimal
- Plugin system — single focused tool, not extensible
- Graphical UI — terminal only
- Configuration file complexity — sensible defaults
- Copy/paste advanced features — basic editing only

## Context

- **User motivation**: Current git editors feel too heavy; want speed and simplicity
- **Monorepo structure**: Building as `crates/gitmedit` in Rust workspace root. Prepared for future git tools but focused on this editor for v1
- **Git context awareness**: Must handle multiple git scenarios (standard commits, merges with conflict markers, rebase todo files, squash contexts)
- **Existing code**: Initial Rust project exists; will use as reference but rewrite cleanly for this design

## Constraints

- **Language**: Rust (monorepo uses Cargo workspace)
- **UI Library**: TUI library choice (crossterm or termion for cross-platform terminal support)
- **Platform**: Linux, macOS (stretch: Windows support)
- **Performance**: Must start faster than vim/nano alternatives
- **Git integration**: Must respect git's file formats exactly (no corruption risk)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Monorepo structure (Cargo workspace) | Prepare for future git tools while keeping focused on gitmedit | — Pending |
| Single crate for v1 (no extraction to core/ui libs yet) | Keep complexity low, extract shared code as tools grow | — Pending |
| Standard hotkeys (Ctrl+S, Esc) vs nano-style (Ctrl+X, Ctrl+C) | Users expect standard shortcuts; simplifies UI | — Pending |
| Minimal status bar (no visible hotkey help initially) | Matches minimal UI philosophy; hotkeys shown on demand | — Pending |

---
*Last updated: 2026-03-18 after Phase 01 completion*
