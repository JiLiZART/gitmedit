# Phase 3: Commit Message Intelligence - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

The editor actively guides users toward well-formed commit messages via a real-time subject line counter with 50/72 color coding, blank line enforcement between subject and body, and an accessible hotkey help overlay that shows only actions relevant to the current editing context.

</domain>

<decisions>
## Implementation Decisions

### Subject Line Counter

- Display in status bar at bottom (follows nano convention, doesn't distract from editing)
- Real-time character count shows current subject line length
- Color coding: green if ≤50 chars, yellow for 51-72, red for >72
- Counter applies only to first line (subject); counts up to first newline
- Updates on every keystroke with no perceivable lag

### Blank Line Enforcement

- Suggest but don't block: visual indicator if blank line between subject and body is missing
- Warning in status bar or prompt (user-visible but non-blocking)
- Allow save even if blank line is missing (user choice to override)
- Detection: at least one completely blank line (no whitespace) between subject and first body line

### Hotkey Help Overlay

- Modal overlay, dismissable with Esc key
- Triggered by Ctrl+H hotkey
- Non-intrusive: renders over editing area without losing editor state
- On dismiss, cursor and scroll position preserved exactly
- Integration: add new `HelpOverlay` state to App state machine

### Context-Aware Action Filtering

- Mode-based filtering: commit mode, merge mode, rebase mode show different action sets
- Commit mode actions: Ctrl+S (save), Esc (cancel), Ctrl+C/X/V (clipboard), Ctrl+U/W/D (editing), Ctrl+H (help)
- Merge mode actions: same as commit, plus note about conflict markers being read-only
- Rebase mode actions: (Phase 4 — don't implement for Phase 3)
- Help text is brief, one-line per action, showing key + description

### Claude's Discretion

- Exact color scheme for status bar counter (beyond green/yellow/red choice)
- Wording of blank line warning message
- Help overlay layout and styling (dialog box, centered, size)
- Help text library (how/where to store help strings)
- Word wrapping behavior for help text in modal

</decisions>

<canonical_refs>
## Canonical References

### Requirements
- `.planning/REQUIREMENTS.md` COMMIT-01 — Subject line character counter displays
- `.planning/REQUIREMENTS.md` COMMIT-02 — Subject line color coding (50/72 thresholds)
- `.planning/REQUIREMENTS.md` COMMIT-03 — Blank line enforcement
- `.planning/REQUIREMENTS.md` COMMIT-06 — Hotkey help on Ctrl+H
- `.planning/REQUIREMENTS.md` HELP-01 through HELP-04 — Help overlay requirements (non-intrusive, mode-aware, dismissable)

### Architecture & Design
- `.planning/research/ARCHITECTURE.md` — Component structure and module organization
- `.planning/research/STACK.md` — Technology choices (ratatui 0.30, crossterm 0.29)

### Phase Dependencies
- `.planning/phases/01-git-contract-tui-shell/01-CONTEXT.md` — Terminal safety, exit code contract, panic handling (foundational)
- `.planning/phases/02-text-editing-comment-handling/02-CONTEXT.md` — Document model, ContentLine enum, TextArea integration, rendering patterns

</canonical_refs>

<code_context>
## Existing Code Insights

### Phase 2 Foundation
- `src/app.rs` — `App` struct holds `document: Document`, `textarea: TextArea`, `context: GitContext`. Phase 3 extends `Action` enum to include `Help` variant.
- `src/app.rs` — `apply()` returns `Outcome` enum. Can extend with `ShowHelp` or similar without changing return type.
- `src/renderer.rs` — Status bar rendered at bottom (1 line). Phase 3 updates status bar to show character counter instead of (or in addition to) current status.
- `src/document.rs` — Document has `first_line()` method to extract subject line. Phase 3 uses this for counter updates.
- `src/context.rs` — `GitContext` enum (Commit, Merge, Rebase, Squash, Tag). Phase 3 uses context to filter help actions shown.

### Integration Points
- Status bar: Update renderer to compute counter and display in status bar
- Action enum: Add `Help` action for Ctrl+H hotkey
- Help overlay state: Add to App or create separate HelpState component
- Modal rendering: New function in renderer to draw help overlay on top of content

### Rendering Architecture
- Layout uses `Direction::Vertical` with `Constraint::Min(0)` (content) and `Constraint::Length(1)` (status bar)
- Phase 3 can keep this structure; help overlay renders on top in a centered modal
- No alternate screen (Phase 1 foundational decision) — modal renders into main buffer

</code_context>

<specifics>
## Specific Ideas

- Subject line counter should feel natural and non-intrusive (like VS Code's column indicator) — updates smoothly
- Blank line enforcement should be helpful, not annoying — suggest but don't prevent
- Help overlay should follow nano conventions: simple, text-based, dismissable with Esc
- Git integration context matters: commit vs merge vs rebase have different relevant actions for the user

</specifics>

<deferred>
## Deferred Ideas

- Rebase mode context filtering — Phase 4 (rebase/squash modes implement their own actions)
- Customizable hotkeys for help trigger — Phase 2 uses hardcoded Ctrl+H; config support is v2 (CONFIG-01)
- Syntax highlighting in help text — Phase 3 uses plain text; rich formatting deferred to v2
- Help text localization — English only in v1

</deferred>

---

*Phase: 03-commit-message-intelligence*
*Context gathered: 2026-03-22*
*Auto-selected during discussion phase with --auto flag*
