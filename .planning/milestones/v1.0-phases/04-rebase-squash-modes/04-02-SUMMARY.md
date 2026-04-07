---
phase: 04-rebase-squash-modes
plan: 02
subsystem: ui
tags: [ratatui, tui, rebase, table, keyboard, renderer]

# Dependency graph
requires:
  - phase: 04-01
    provides: RebaseLine/RebaseAction types, App rebase accessors, CycleRebaseAction/MoveRebaseUp/MoveRebaseDown actions
provides:
  - render_rebase_table() in renderer.rs — structured table with colored action cells and highlighted selected row
  - render_rebase_status_bar() in renderer.rs — rebase-specific status bar with Tab/Save/Cancel/Help hints
  - GitContext::Rebase branch in Renderer::render() dispatching to rebase-specific widgets
  - GitContext::Rebase help overlay text with rebase-specific key bindings
  - Three-branch event loop in main.rs: help-visible | rebase-mode | normal-editing
  - Tab -> CycleRebaseAction, Up/Down -> MoveRebaseUp/MoveRebaseDown in rebase mode
affects: [04-03]

# Tech tracking
tech-stack:
  added: [ratatui Table widget, ratatui Cell/Row widgets]
  patterns: [context-dispatch rendering, structured table for rebase-todo, three-branch event loop for mode separation]

key-files:
  created: []
  modified:
    - src/renderer.rs
    - src/main.rs

key-decisions:
  - "Renderer::render() dispatches on GitContext::Rebase before calling content/status functions — clean mode separation without modifying existing render paths"
  - "Scroll offset computed as selected_line_idx.saturating_sub(visible_height/2) — keeps selected row centered when list exceeds screen"
  - "Rebase help overlay returns early with entirely different line set — no base_actions bleed into rebase help"
  - "Event loop branch order: help-visible -> rebase-mode -> normal-editing — ensures help dismissal works in both modes and rebase never delegates to textarea"

patterns-established:
  - "Context-dispatch rendering: check app.context() at top of render(), delegate to context-specific render functions"
  - "Three-branch event loop: is_help_visible() gate first, then context-specific modes, then default editing"

requirements-completed: [REBASE-02, REBASE-03]

# Metrics
duration: 1min
completed: 2026-03-23
---

# Phase 04 Plan 02: Rebase Table Rendering and Keyboard Input Summary

**Ratatui Table-based rebase-todo renderer with per-action color coding, selected-row highlight, and Tab/arrow key navigation wired in a three-branch event loop.**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-23T11:47:46Z
- **Completed:** 2026-03-23T11:48:46Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Rebase mode renders a structured three-column table (action | hash | subject) with per-action colors: green=pick, yellow=squash, cyan=fixup, red=drop, magenta=exec
- Selected row is visually highlighted with DarkGray background and BOLD modifier; scroll offset keeps selected row visible when list exceeds terminal height
- Rebase-specific status bar shows "Line N/Total | Tab Cycle action | ^S Save | Esc Cancel | ^H Help"
- Rebase-specific help overlay shows Tab/Up/Down/Ctrl+S/Esc with notes about comment and exec lines
- Event loop gains a rebase mode branch that handles Tab->CycleRebaseAction, Up->MoveRebaseUp, Down->MoveRebaseDown, Ctrl+S save, Esc cancel — no free-text input delegated to textarea

## Task Commits

Each task was committed atomically:

1. **Task 1: Add rebase table rendering and rebase-aware status bar/help to renderer.rs** - `ae5da02` (feat)
2. **Task 2: Wire Tab and arrow keys to rebase actions in main.rs event loop** - `60c2b76` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `src/renderer.rs` - Added render_rebase_table(), render_rebase_status_bar(), GitContext::Rebase branch in render() and help_text_for_context()
- `src/main.rs` - Added GitContext import and rebase mode branch in event loop with Tab/Up/Down/Ctrl+S/Esc handlers

## Decisions Made
- Renderer::render() dispatches on GitContext::Rebase to call render_rebase_table()/render_rebase_status_bar() instead of the existing text-editor widgets — avoids any interplay between textarea and rebase display
- Scroll offset is computed as `selected_line_idx.saturating_sub(visible_height / 2)` using `.skip()` and `.take()` on the row iterator — avoids needing ratatui TableState for basic scroll behavior
- Help overlay for Rebase context returns early with a completely different vec — no base_actions (clipboard, undo, word delete) appear in rebase help since TextArea is not active
- Event loop branch order is: help_visible -> rebase_mode -> normal_editing — guarantees help dismissal (Esc/Ctrl+H) works identically in both modes

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Rebase mode is now visually functional and interactive: `git rebase -i` with gitmedit displays a usable structured table
- Tab cycles actions, Up/Down navigate commits, Ctrl+S saves the plan, Esc cancels
- Plan 04-03 can now add squash-context detection and any remaining rebase/squash polish

---
*Phase: 04-rebase-squash-modes*
*Completed: 2026-03-23*
