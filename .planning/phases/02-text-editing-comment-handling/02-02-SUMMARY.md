---
phase: 02-text-editing-comment-handling
plan: "02"
subsystem: ui
tags: [ratatui, ratatui-textarea, crossterm, rust, tui, editor]

# Dependency graph
requires:
  - phase: 02-01
    provides: Document struct with ContentLine enum, editable_lines(), serialize(), full_row_for_editable()
provides:
  - App struct backed by Document + TextArea<'static> with serialized_content() for save
  - Event loop delegating all non-Ctrl+S/Esc key presses to textarea_mut().input()
  - Renderer with per-line styling: Comment=DarkGray, ConflictMarker=Red/White, Content=default+underline on cursor
  - Scroll viewport that tracks textarea cursor position
  - Blinking terminal cursor positioned via frame.set_cursor_position()
affects: [03-ux-polish, 04-rebase-squash-support]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "App owns Document (parse model) + TextArea (editing model) — separated concerns"
    - "Renderer iterates full document lines, tracks editable_idx separately to sync with TextArea content"
    - "Event loop intercepts Ctrl+S/Esc; all other presses delegated to TextArea.input()"
    - "Serialization via Document::serialize(textarea.lines()) — merges edits back preserving comments"

key-files:
  created: []
  modified:
    - src/app.rs
    - src/main.rs
    - src/renderer.rs

key-decisions:
  - "Renderer does NOT use frame.render_widget(&textarea, area) — builds custom Line spans for per-line styling"
  - "cursor_full_row uses usize::MAX sentinel when no editable lines exist (avoids Option overhead)"
  - "Event::Resize handled as explicit no-op — ratatui terminal.draw() calls autoresize() automatically"
  - "Renderer and App updated together in single compilation pass — renderer needed App's new API to compile"

patterns-established:
  - "Scroll offset = cursor_full_row - visible_height + 1 when cursor beyond viewport bottom"
  - "editable_idx tracked independently during line iteration to map full-document rows to TextArea content"

requirements-completed:
  - CTX-05
  - EDIT-01
  - EDIT-02
  - EDIT-04
  - EDIT-05
  - EDIT-06
  - MERGE-02
  - MERGE-03
  - COMMIT-04
  - COMMIT-05

# Metrics
duration: 5min
completed: 2026-03-20
---

# Phase 02 Plan 02: Text Editing + Styled Rendering Summary

**App refactored to Document+TextArea with per-line Comment/ConflictMarker/Content styling, scroll viewport, and full TextArea key delegation via crossterm event passthrough**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-20T12:37:18Z
- **Completed:** 2026-03-20T12:39:17Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- App struct now holds `Document` (parse model) and `TextArea<'static>` (editing model) instead of raw `String`
- TextArea seeded with only editable Content lines; `serialized_content()` merges edits back via `Document::serialize()`
- Event loop delegates all non-intercepted key presses to `textarea_mut().input()`, enabling arrow keys, insertion, backspace, delete, Home/End, Enter
- Renderer replaced with per-line styled output: Comment lines in DarkGray, ConflictMarker lines in Red/White, Content lines with underline on the active cursor line
- Scroll viewport follows the textarea cursor, `frame.set_cursor_position()` places blinking terminal cursor correctly
- All 33 tests pass; `cargo build` and `cargo check` clean (warnings only)

## Task Commits

1. **Task 1: Refactor App with Document+TextArea, update event loop** - `8c0567e` (feat)
2. **Task 2: Upgrade Renderer with per-line Content/Comment/ConflictMarker styling** - `b17c571` (feat)

## Files Created/Modified

- `src/app.rs` - Replaced `content: String` with `document: Document` + `textarea: TextArea<'static>`; added `serialized_content()`, `textarea()`, `textarea_mut()`, `document()`; updated tests
- `src/main.rs` - Event loop updated: passes raw content to `App::new()`, delegates key presses to `textarea_mut().input()`, uses `serialized_content()` for save
- `src/renderer.rs` - Full rewrite: per-line styling for all three ContentLine variants, scroll viewport, `set_cursor_position()`, `Wrap { trim: false }`

## Decisions Made

- Renderer does NOT use `frame.render_widget(&textarea, area)` — builds custom `Line` spans so each ContentLine variant can have independent styling
- `usize::MAX` used as sentinel for "no cursor" when there are no editable lines (avoids wrapping in `Option<usize>` for what is a UI rendering concern)
- `Event::Resize` handled as an explicit no-op arm — ratatui's `terminal.draw()` calls `autoresize()` so no manual resize handling is needed
- Renderer updated in same execution pass as App (not deferred to Task 2 commit) because the old `app.content()` call in renderer.rs caused a compile error that blocked Task 1 verification

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Renderer updated alongside App refactor in single compilation pass**
- **Found during:** Task 1 (App refactor)
- **Issue:** The existing `src/renderer.rs` called `app.content()` which no longer exists after removing `content: String`. This caused a compile error that prevented `cargo test` from passing for Task 1.
- **Fix:** Implemented the full Task 2 renderer rewrite during the same compilation cycle as Task 1. The tasks were committed separately (`8c0567e` for app+main, `b17c571` for renderer) to preserve atomicity semantics.
- **Files modified:** src/renderer.rs
- **Verification:** `cargo test` passes 33/33 after both files written; `cargo build` clean
- **Committed in:** b17c571 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking — compile dependency between tasks)
**Impact on plan:** Auto-fix was necessary — the two tasks have a hard compile-time dependency. Commits remain separate and atomic. No scope creep.

## Issues Encountered

None beyond the compile-time dependency between tasks documented above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Full text editing stack is operational: Document parsing, TextArea editing, styled rendering, and save/cancel all work end-to-end
- Plan 02-03 can proceed: UX polish, line wrapping visual behavior, and any remaining keyboard shortcut work
- No blockers

---
*Phase: 02-text-editing-comment-handling*
*Completed: 2026-03-20*
