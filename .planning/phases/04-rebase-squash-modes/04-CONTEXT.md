# Phase 4: Rebase + Squash Modes - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

gitmedit handles interactive rebase and squash operations as a sequence.editor replacement — parsing git-rebase-todo into a structured table, supporting action cycling (pick→squash→fixup→drop), and rendering squash commit logs as read-only context alongside the editable combined message.

</domain>

<decisions>
## Implementation Decisions

### Rebase Todo Parsing & Display

- Parse git-rebase-todo line-by-line into structured `RebaseLine` enum with action + commit hash + message variants
- Display as a structured table (not free-form text): columns for action, abbreviated hash, and subject
- Comment lines are preserved and displayed but not editable (same `ContentLine` pattern from Phase 2)
- Action names: pick (p), reword (r), squash (s), fixup (f), drop (d), exec (x) — all git-standard abbreviations supported

### Action Cycling Hotkey

- Hotkey: Tab key cycles through action types (pick→squash→fixup→drop→pick)
- Only non-comment lines can cycle; comment lines are read-only
- Selected line is visually highlighted (inverse/bold style)
- Display shows "(Tab to cycle)" help text in status bar during rebase mode

### Rebase File Writing

- On save, reconstruct rebase-todo file in exact git format: `{action} {hash} {subject}` per line
- Comment lines written back byte-for-byte unchanged
- No extra whitespace, no reformatting — git accepts file exactly as written
- Atomic write: write-to-temp-then-rename (established pattern from Phase 1)

### Squash Context Display

- Detect SQUASH_MSG file (git-generated "This is a combination of N commits" block)
- Parse accumulated commit log (read-only, styled section at top or side)
- Editable area below for combined commit message (normal editing applies)
- Commit log marked as protected (cannot delete, cannot edit)
- Visual distinction: different background color (dark gray or muted) for commit log section

### Mode Detection & Switching

- Existing context detection (Phase 1) already identifies git-rebase-todo and SQUASH_MSG files
- Add `GitContext::Rebase` and `GitContext::Squash` variants
- UI switches to structured rebase table view for rebase-todo (not text editing)
- For SQUASH_MSG, use dual-pane or stacked layout: read-only log + editable message

### Claude's Discretion

- Exact table column widths and alignment for rebase display
- Visual styling colors for protected sections and highlighted lines
- Cursor movement in table (arrow keys move between rows vs columns)
- Tab vs other hotkey for action cycling (Tab chosen for ergonomics)
- Line wrapping behavior for long commit subjects in table view
- How to handle rebase-todo files larger than screen height (scrolling strategy)

</decisions>

<canonical_refs>
## Canonical References

### Requirements
- `.planning/REQUIREMENTS.md` REBASE-01 through REBASE-06 — Rebase todo parsing, action cycling, file writing
- `.planning/REQUIREMENTS.md` SQUASH-01 through SQUASH-04 — Squash detection, commit log display, message editing

### Architecture & Design
- `.planning/research/ARCHITECTURE.md` — Component structure and module organization
- `.planning/research/STACK.md` — Technology choices (ratatui 0.30, crossterm 0.29)

### Phase Dependencies
- `.planning/phases/02-text-editing-comment-handling/02-CONTEXT.md` — ContentLine enum, comment handling patterns, editor state architecture
- `.planning/phases/03-commit-message-intelligence/03-CONTEXT.md` — GitContext enum, rendering patterns, modal overlay patterns

### Key Git Specifications
- Git rebase-todo format: line-by-line `{action} {hash} {subject}`, comment lines start with `#`, exact format must be preserved
- SQUASH_MSG format: git-generated "This is a combination of N commits" block followed by editable message

</canonical_refs>

<code_context>
## Existing Code Insights

### Phase 2-3 Foundation
- `GitContext` enum exists with Commit, Merge, Rebase (TODO), Squash (TODO), Tag variants
- `ContentLine` enum (Content, Comment, ConflictMarker) can be reused for rebase-todo parsing
- `Document` struct can be extended with rebase-specific parsing
- `Renderer` has established patterns for styled output; modal/overlay rendering can be adapted for squash context display
- `App::apply()` returns `Outcome` enum; can extend `Action` enum for Tab cycling

### Rendering Architecture
- No alternate screen (foundational Phase 1 constraint applies)
- Immediate-mode rendering with ratatui Layout — can use Table widget for rebase display
- Status bar pattern established (used for counter/warnings in Phase 3)

### Integration Points
- Context detection (Phase 1) already reads filenames; Rebase/Squash contexts already partially wired
- Event loop (main.rs) handles keyboard input; Tab key can be added as new action
- File writing infrastructure (Phase 1 atomic write) can be reused for rebase-todo output

</code_context>

<specifics>
## Specific Ideas

- Rebase table should feel lightweight and quick — users expect vim-style action cycling (Tab key)
- Squash context display should follow nano convention — read-only section clearly marked, editing area separated
- Action cycling should wrap around (drop→pick) for ergonomics
- Error recovery: if rebase-todo is malformed, handle gracefully (show raw text with warning, allow editing)

</specifics>

<deferred>
## Deferred Ideas

- Line reordering in rebase (move commits up/down) — v2 feature (high interaction complexity)
- Squash message templates — v2 feature
- Rebase-specific hotkey help overlay — Phase 4 can reuse Phase 3 help (update context filtering)
- Exec line support with arguments — v2 enhancement

</deferred>

---

*Phase: 04-rebase-squash-modes*
*Context gathered: 2026-03-22*
*Auto-selected during discussion phase with --auto flag*
