# Phase 2: Text Editing + Comment Handling - Context

**Gathered:** 2026-03-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can fully edit commit messages and merge messages, with comment lines visually distinct and preserved verbatim, conflict markers styled and protected, and save/cancel hotkeys functional. Multiline messages, character-by-character editing, undo/redo, and 30+ FPS rendering are required.

</domain>

<decisions>
## Implementation Decisions

### Comment Handling Strategy

- **Parse on load**: Comments detected at startup, not during runtime editing
- **Storage model**: `Vec<ContentLine>` enum where each line is `Content(String)` or `Comment(String)`, preserving order and structure
- **Comment character**: Read `core.commentChar` from git config (default `#`) at startup; supports user customization per repository
- **Rendering**: Comment lines displayed in different foreground color (e.g., DarkGray) to signal read-only status
- **Preservation**: Comment lines written back byte-for-byte on save; no modification of comment text

### Text Model Architecture

- **Internal representation**: `Vec<ContentLine>` enum, not single String or line metadata
- **Cursor tracking**: Line and column coordinates (line: usize, col: usize) for natural editor semantics
- **Undo/Redo**: Command history pattern — each keystroke is stored as an Action, undo/redo rewind/replay from history
- **Mutation safety**: Cursor can move to any line (including comments) for visual feedback, but mutations fail if target line is a comment. No accidental comment destruction.

### Merge Conflict Handling

- **Conflict block representation**: Separate `ContentLine::ConflictMarker` variant for conflict markers (<<<<<<, ======, >>>>>>>)
- **Parsing strategy**: Line-by-line detection; each marker line identified individually, not as a compound block structure
- **Editability**: Conflict marker lines (<<<<<<, ======, >>>>>>) are read-only. Content lines between markers (both left/right sides and final resolution) are fully editable.
- **Styling**: Conflict marker lines displayed with different background color to distinguish them visually from comments and content
- **Preservation**: Markers written back exactly as read; resolution text changes preserved

### Text Editing Operations

- **Basic operations**: Insert character, delete character (backspace/delete), arrow key navigation, Home/End for line start/end, Ctrl+U to delete entire line
- **Word operations**: Ctrl+W to delete previous word, Ctrl+D to delete next word
- **System clipboard**: Ctrl+C to copy, Ctrl+X to cut, Ctrl+V to paste from system clipboard (requires arboard or similar dependency)
- **Text selection**: Shift+arrow keys (left/right/up/down) to select text; required for clipboard operations
- **Line wrapping**: Hard wrap at terminal width during rendering only; underlying newline structure preserved, no horizontal scroll

### Rendering & Performance

- **Update rate**: Target 30+ FPS during typing (no perceived lag)
- **Cursor rendering**: Visible cursor indicator at (line, col) position
- **Large files**: No noticeable lag on files >10KB

### Claude's Discretion

- Exact styling colors for comments vs conflict markers (beyond background/foreground choice)
- Word boundary detection algorithm (what constitutes a "word" for Ctrl+W/D)
- Clipboard provider (arboard vs copypasta vs other crate)
- How to handle terminal resize mid-editing
- Exact panic/error handling for clipboard unavailability

</decisions>

<canonical_refs>
## Canonical References

### Requirements
- `.planning/REQUIREMENTS.md` CTX-03 through CTX-06 — Comment character config, parsing, storage, preservation
- `.planning/REQUIREMENTS.md` EDIT-01 through EDIT-07 — Text editing operations, movement, deletion, multiline, wrapping, undo
- `.planning/REQUIREMENTS.md` COMMIT-04, COMMIT-05 — Save/cancel hotkeys (already wired in Phase 1)
- `.planning/REQUIREMENTS.md` MERGE-01 through MERGE-03 — Merge conflict marker styling, protection, editability
- `.planning/REQUIREMENTS.md` PERF-02, PERF-03 — 30+ FPS rendering, no lag on large files

### Architecture & Design
- `.planning/research/ARCHITECTURE.md` — Component structure and data flow from Phase 1 research
- `.planning/research/STACK.md` — Technology choices (ratatui, crossterm); dependency minimalism principles

### Phase Dependencies
- `.planning/phases/01-git-contract-tui-shell/01-CONTEXT.md` — Phase 1 decisions on terminal safety, raw mode, panic handling, exit codes (foundational)

</canonical_refs>

<code_context>
## Existing Code Insights

### Phase 1 Foundation
- `src/app.rs` — App struct holds `content: String` and `context: GitContext`. Phase 2 refactors to hold `Vec<ContentLine>` instead.
- `src/app.rs` — `apply()` method returns `Outcome` enum. Can be extended to handle Edit actions without changing return type.
- `src/renderer.rs` — Currently renders raw text lines. Phase 2 upgrades to render Content/Comment/ConflictMarker variants with appropriate styling.
- `src/main.rs` — Event loop wired for Ctrl+S (save) and Esc (cancel). Phase 2 expands to handle arrow keys, Ctrl+U, Ctrl+W/D, Ctrl+C/V, Shift+arrows.

### Integration Points
- No clipboard dependency exists yet; Phase 2 adds arboard (or equivalent)
- Terminal resizing (SIGWINCH) not yet handled; Phase 2 can defer or add simple handling
- Core.commentChar reading requires spawning subprocess or reading .git/config; Phase 2 research determines approach

### Dependencies to Add
- `arboard` or `copypasta` for system clipboard
- Potentially `regex` for word boundary detection (or implement simple heuristic)

</code_context>

<specifics>
## Specific Ideas

- Comment behavior should match git's own display — subtle but clear that they're not part of the commit message
- Merge conflict handling should prioritize safety: user can easily see and navigate around markers but can't accidentally destroy them
- Keyboard shortcuts should follow standard editor conventions (arrows, Ctrl+U, Ctrl+W) so user knowledge transfers from nano/emacs/vim muscle memory

</specifics>

<deferred>
## Deferred Ideas

- Line reordering in merge conflicts (move <<<< to choose sides) — Phase 2.1 enhancement if needed
- Regex-based search/replace — v2 feature per REQUIREMENTS.md (EDIT-10)
- Terminal resize handling — can defer graceful handling to Phase 3 if basic resizing works
- Customizable hotkeys — Phase 2 uses hardcoded shortcuts; config file support is v2 (CONFIG-01)

</deferred>

---

*Phase: 02-text-editing-comment-handling*
*Context gathered: 2026-03-19*
