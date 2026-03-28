# Phase 6: Rebase View — Full Commit Subject Visibility - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Make long commit subjects fully visible in the rebase table by wrapping them to continuation lines instead of letting ratatui's Table widget silently truncate them. No horizontal scrolling — text wraps within the terminal width.

</domain>

<decisions>
## Implementation Decisions

### Wrapping Strategy
- **D-01:** No horizontal scrolling. Long commit subjects wrap to the next line instead of being truncated.
- **D-02:** Continuation lines indent to align under the subject column (past the action + hash columns), keeping the table visually structured.
- **D-03:** Left/right arrow keys remain unused in rebase mode (no new keybindings needed).

### Rendering
- **D-04:** The subject column (`Constraint::Min(20)`) wrapping must account for the 8+8=16 chars consumed by the action and hash columns, so wrap width = terminal_width - 16.
- **D-05:** Vertical scroll offset calculation must account for wrapped rows consuming multiple lines of visible height.

### Claude's Discretion
- Exact method to achieve wrapping within ratatui's Table (Row height, manual line-breaking before render, or switching from Table to manual Line rendering)
- Whether to add visual cues for wrapped rows (e.g., no-wrap indicator vs subtle continuation marker)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` REBASE-01 through REBASE-06 — Rebase todo parsing, display, and file writing constraints

### Phase Dependencies
- `.planning/phases/04-rebase-squash-modes/04-CONTEXT.md` — Rebase table architecture, rendering decisions, action cycling
- `src/renderer.rs` lines 314-378 — Current `render_rebase_table()` implementation (truncation happens here)
- `src/renderer.rs` lines 66-98 — Existing horizontal scroll logic in text editor mode (reference for scroll_offset patterns, though not directly reused)

### Key Constraints
- No alternate screen (IO-06) — wrapping must work in main terminal buffer
- ratatui `Table` widget silently truncates cell content exceeding column width — this is the root cause being fixed

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `render_rebase_table()` in `src/renderer.rs:314` — the function to modify; currently uses `Table::new(rows, widths)` with `Constraint::Min(20)` for subject
- `Row::new(vec![...])` — ratatui Row supports `.height(n)` to render multi-line rows
- Vertical scroll logic (`scroll_offset` calculation at line 320) — needs adjustment to account for variable row heights

### Established Patterns
- Table uses fixed `Constraint::Length(8)` for action and hash columns, `Constraint::Min(20)` for subject
- Selected row highlighted with `Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)`
- Comment rows span full width as single Cell

### Integration Points
- `render_rebase_table()` is the only function that needs changes (rendering)
- Vertical scroll offset calculation must be updated to count wrapped lines, not just row count
- No changes to parsing, event handling, or file writing

</code_context>

<specifics>
## Specific Ideas

- Continuation lines should look like:
  ```
  pick    abc1234 Fix the very long commit
                  subject that wraps here
  pick    def5678 Short subject
  ```
- The indent on continuation lines = 16 chars (8 for action + 8 for hash) to align under subject start

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 06-rebase-view-horizontal-scrolling*
*Context gathered: 2026-03-28*
