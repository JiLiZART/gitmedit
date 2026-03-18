# gitmedit

## What This Is

A lightweight TUI git editor that replaces heavier defaults (vim, nano) as the global git editor. Set it once via `git config --global core.editor gitmedit` and it handles commits, merges, rebases, and squashes with a minimal interface and fast startup.

## Core Value

Provide a fast, distraction-free git editor that feels like nano's simplicity but understands git's context (merges, rebases, squashes) without bloat.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] User can set gitmedit as global git editor
- [ ] User sees TUI window with editable message area
- [ ] User can commit with standard hotkey (Ctrl+S save)
- [ ] User can cancel edits with standard hotkey (Esc cancel)
- [ ] Editor displays available hotkeys (non-intrusive)
- [ ] Editor handles commit message files (COMMIT_EDITMSG)
- [ ] Editor handles merge conflict messages (MERGE_MSG)
- [ ] Editor handles rebase todo files with comment detection
- [ ] Editor detects squash context and highlights commit log
- [ ] Editor provides hotkeys to manipulate squash commits (mark/delete/reorder)
- [ ] Multiline commit messages work correctly (paragraphs with blank lines)
- [ ] Empty message is allowed to cancel but warns user
- [ ] Comments in files are preserved and styled (not editable)
- [ ] Exit status codes match git expectations (success/cancel)

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
*Last updated: 2026-03-18 after initialization*
