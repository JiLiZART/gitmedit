---
status: awaiting_human_verify
trigger: "Investigate table text wrapping/overflow issue in rebase file editing (Phase 04 implementation)"
created: 2026-03-26T00:00:00Z
updated: 2026-03-26T00:00:00Z
---

## Current Focus

hypothesis: User confirmed text truncation. Long commit subjects are cut off at cell width boundary.
Ratatui Table widget truncates (not wraps) text exceeding cell width. No explicit truncation logic in code.
Subject cell uses Constraint::Min(20) which allocates remaining terminal width, but ratatui silently truncates
any text exceeding that width. Solution: either truncate explicitly with indicator, or switch to List widget
for better scrolling/display options, or implement multi-line row height for Table.

test: Compare rendering behavior between Table (current) and List widget (alternative)

expecting: Need to evaluate which solution best preserves usability and code complexity

next_action: Research List widget capabilities for horizontal scrolling and multi-line content

## Symptoms

expected: Text in rebase table should display clearly even with long commit subject lines
actual: Text wrapping/overflow occurs in rebase table — long lines don't display correctly in table cells
errors: None reported; feature "works" but has rendering issues
reproduction: Open rebase file with long commit subject lines
started: Since Phase 04 implementation (never worked properly with text wrapping)

## Eliminated

(none yet)

## Evidence

- timestamp: 2026-03-26
  checked: render_rebase_table function (lines 314-378)
  found: |
    Table uses three column constraints:
    - Constraint::Length(8) for action column
    - Constraint::Length(8) for hash column
    - Constraint::Min(20) for subject column

    Rows are built with Cell::from(subject.clone()) without any truncation or wrapping logic.
    The table widget receives the full subject text with no length limiting.
  implication: Subject cells may exceed available width, causing overflow/wrapping issues in ratatui Table

- timestamp: 2026-03-26
  checked: ratatui widget documentation (web search)
  found: |
    - ratatui Table widget does NOT support text wrapping (only Paragraph supports wrapping)
    - ratatui List widget also does NOT have built-in text wrapping
    - List widget does support multi-line items but has known issues with scroll/rendering
    - Both widgets truncate text that exceeds available width
  implication: Table truncation happens silently; user doesn't see full subject lines

- timestamp: 2026-03-26
  checked: constraint configuration in render_rebase_table
  found: |
    Constraint::Min(20) for subject column means the subject column GROWS to fill available space,
    but the Table widget itself does not wrap text within a cell.
    If terminal is 120 cols wide: action=8, hash=8, subject gets ~104 cols.
    But the Table widget will just truncate any text beyond the cell boundary.
  implication: User cannot see full subject lines if they exceed column width

- timestamp: 2026-03-26
  checked: Row height mechanism in ratatui Table
  found: |
    ratatui Row struct supports a .height(n) method to span multiple terminal lines.
    Can insert newline characters (\n) in cell content and set row height accordingly.
    This allows multi-line cell content within a Table widget without switching to List.
    Current code: Row::new(...) creates rows with default height (1 line per row).
    This means any text with newlines is truncated, OR text that exceeds column width is silently cut off.
  implication: Problem is NOT that Table can't handle wrapping—it CAN via Row.height().
             The current code just doesn't use this feature.

- timestamp: 2026-03-26
  checked: Scroll offset calculation in render_rebase_table (lines 318-328)
  found: |
    Code assumes each row is exactly 1 terminal line:
      visible_height = area.height as usize
      scroll_offset = idx.saturating_sub(visible_height / 2)
      skip(scroll_offset).take(visible_height)

    But if Table widget auto-wraps long text and creates multi-line rows,
    then some rows occupy 2+ terminal lines, breaking this calculation.
    The skip() and take() count ROWS, not terminal lines.
    With multi-line rows, displayed area doesn't match visible_height.
  implication: Scroll logic needs adjustment if we implement multi-line rows.

- timestamp: 2026-03-26
  checked: User confirmation from checkpoint
  found: |
    User confirmed: Text Truncation (Option A) — Long commit subjects are cut off and cannot be seen fully.
    This is SILENT truncation, not wrapping or overlapping.
    Visual behavior: "pick abc1234 This is a very long commit subject that..." → "pick abc1234 This is a very "
  implication: ROOT CAUSE CONFIRMED: ratatui Table silently truncates text exceeding cell width.
             No text wrapping occurs. Cells simply cut off text at boundary.
             Solution: must either (1) use explicit truncation with indicator like "..."
             or (2) implement horizontal scrolling within cell
             or (3) implement multi-line rows with wrapping
             or (4) switch to different widget

- timestamp: 2026-03-26
  checked: ratatui List widget capabilities
  found: |
    - List widget has multi-line item support with repeat_highlight_symbol
    - Recent PR #1553 (2025) improved List.offset scrolling for multi-line items
    - List widget is designed for vertical scrolling only, NOT horizontal
    - Multi-line items in List work by setting item's display to multiple lines,
      not by horizontal wrapping of single-line items
    - tui-widget-list crate offers ListView with optional horizontal scrolling,
      but requires additional dependency
  implication: Switching to List doesn't solve horizontal scrolling problem.
             Multi-line display in List means each commit would take 2+ rows
             (still doesn't show full subject on one line).
             Would need tui-widget-list for true horizontal scroll.

- timestamp: 2026-03-26
  checked: ratatui Table widget horizontal scrolling options
  found: |
    - Table widget truncates longer cells silently (no built-in wrapping)
    - RFC #174 discusses scrollable widgets but horizontal scroll for Table is NOT implemented
    - Table can use Scrollbar widget (visual only, doesn't affect column position)
    - Issue #1488 requests scroll_padding for Table (like List has)
    - Horizontal scrolling by cells is NOT a feature of Table
    - Solution options: (1) implement manual horizontal scroll state
                        (2) truncate with ellipsis indicator
                        (3) use multi-line rows with wrapping (breaks scroll calculation)
  implication: Table widget does NOT support horizontal scrolling natively.
             To show full subjects, must either:
             A) Implement custom horizontal scroll state (most work, best UX)
             B) Truncate with ellipsis like "..." (simple, but lossy)
             C) Switch widget (List needs wrapping to 2+ lines; tui-widget-list needs new dep)

## Resolution

root_cause: ratatui Table widget truncates cell content that exceeds column width. No text wrapping occurs.
            Subject column uses Constraint::Min(20) which allocates remaining terminal space, but Table
            silently truncates any text exceeding that width. Subject cells receive full text but Table
            renders only the portion that fits within the cell boundary.

fix: Recommend Option A — Implement horizontal scrolling for subject column in Table widget.
     This requires: (1) Add horizontal scroll state to App
                    (2) Track current scroll offset per row
                    (3) Implement scroll_line() logic in render_rebase_table for subject column
                    (4) Add arrow keys or arrow hints to indicate scrollable content
     Complexity: Medium (requires App state change, scroll tracking in renderer)

     Alternative Option B (simpler but lossy): Truncate subjects with ellipsis indicator
     - Modify line 354 to truncate subject to available width with "..." suffix
     - Complexity: Low (one-line fix)
     - Trade-off: Users can't see full subjects, only hints

verification: (pending user decision on solution)
files_changed: []
