# Phase 6: Rebase View — Full Commit Subject Visibility - Research

**Researched:** 2026-03-30
**Domain:** ratatui Table / Row multi-line rendering, Rust string wrapping
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** No horizontal scrolling. Long commit subjects wrap to the next line instead of being truncated.
- **D-02:** Continuation lines indent to align under the subject column (past the action + hash columns), keeping the table visually structured.
- **D-03:** Left/right arrow keys remain unused in rebase mode (no new keybindings needed).
- **D-04:** The subject column (`Constraint::Min(20)`) wrapping must account for the 8+8=16 chars consumed by the action and hash columns, so wrap width = terminal_width - 16.
- **D-05:** Vertical scroll offset calculation must account for wrapped rows consuming multiple lines of visible height.

### Claude's Discretion
- Exact method to achieve wrapping within ratatui's Table (Row height, manual line-breaking before render, or switching from Table to manual Line rendering)
- Whether to add visual cues for wrapped rows (e.g., no-wrap indicator vs subtle continuation marker)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

## Summary

Phase 6 is a targeted rendering improvement to `render_rebase_table()` in `src/renderer.rs`. The root problem is that ratatui's `Table` widget silently truncates `Cell` content when it exceeds the computed column width. The fix is to pre-compute subject line-breaks in Rust before handing content to ratatui, then pass a multi-line `Text` to the subject `Cell` paired with `Row::height(n)` to reserve the extra terminal lines.

The implementation is self-contained: no parsing changes, no event-handling changes, no new app state. The only behavioral change is that the vertical scroll offset calculation must switch from "count of rows" to "count of terminal lines consumed by visible rows" since wrapped rows occupy more than one line each.

**Primary recommendation:** Pre-break subjects in Rust using a `wrap_subject(subject, wrap_width)` helper, build a multi-line `Text` for the subject `Cell`, set `Row::height(n)` where n = wrapped line count, and update scroll accounting to sum `row.height_consumed` instead of row count.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ratatui | 0.30.0 | TUI rendering, Table/Row/Cell widgets | Already in use |
| Rust std | — | String slicing, char-boundary safety | No deps needed |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| unicode-width | (transitive via ratatui) | Display width of Unicode chars | Only needed if wide-char support required beyond ASCII; not needed for phase scope |

**Installation:** No new dependencies required.

## Architecture Patterns

### Recommended Project Structure
No structural changes. All changes are within:
```
src/
└── renderer.rs    # render_rebase_table() — the only function modified
```

### Pattern 1: Pre-break Subject Before Render (chosen approach)

**What:** Before constructing `Row`, split the subject string into wrapped lines in Rust. Pass a multi-line `Text` to the subject `Cell`. Set `Row::height(n)` where `n` is the number of wrapped lines.

**When to use:** Whenever ratatui `Table` must display text longer than the column width. ratatui does NOT auto-wrap cell content.

**Key insight from source inspection:** `Text::raw("line1\nline2")` correctly splits into two `Line` entries (via `str::lines()`). `Row::height(2)` causes ratatui to render both lines within the row's allocated terminal space. If `height` is not set to match the line count, any extra lines are truncated.

**Example:**
```rust
// Source: ratatui-0.29.0/src/widgets/table/row.rs (lines 151-162)
// Source: ratatui-0.29.0/src/text/text.rs (lines 240-246)

fn wrap_subject(subject: &str, wrap_width: usize) -> Vec<String> {
    if wrap_width == 0 || subject.len() <= wrap_width {
        return vec![subject.to_string()];
    }
    let mut lines = Vec::new();
    let indent = " ".repeat(16); // align continuation under subject column
    let mut remaining = subject;
    let mut first = true;
    while !remaining.is_empty() {
        let available = if first { wrap_width } else { wrap_width }; // same width
        // Break at char boundary, not byte boundary
        let take = remaining
            .char_indices()
            .take_while(|(i, _)| *i < available)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(remaining.len());
        let (chunk, rest) = remaining.split_at(take);
        if first {
            lines.push(chunk.to_string());
            first = false;
        } else {
            lines.push(format!("{}{}", indent, chunk.trim_start()));
        }
        remaining = rest.trim_start();
    }
    lines
}

// Inside render_rebase_table():
let wrap_width = area.width.saturating_sub(16) as usize;
let wrapped_lines = wrap_subject(subject, wrap_width);
let height = wrapped_lines.len() as u16;
let subject_text = wrapped_lines.join("\n");

Row::new(vec![
    Cell::from(format!("{:<7}", action.as_str())).style(action_style),
    Cell::from(hash.chars().take(7).collect::<String>()),
    Cell::from(subject_text),
])
.height(height)
.style(row_style)
```

**Why `wrap_width = area.width - 16`:** The table layout allocates `Constraint::Length(8)` twice (action + hash) = 16 chars consumed. The subject column gets `Constraint::Min(20)` which resolves to `area.width - 16` in practice. Passing `area.width` directly avoids the need to query ratatui's resolved constraint width.

**Note on `area.width` availability:** `area: Rect` is already a parameter to `render_rebase_table()` — `area.width` is immediately accessible.

### Pattern 2: Scroll Offset Accounting for Variable Row Heights

**What:** The current scroll calculation counts rows (1 row = 1 terminal line). With wrapped rows this breaks. The new calculation must count terminal lines consumed.

**When to use:** Any time `Row::height(n > 1)` is used in a manually-scrolled table.

**Example:**
```rust
// Compute per-row line heights BEFORE building the display rows
let row_heights: Vec<usize> = rebase_lines
    .iter()
    .map(|line| match line {
        RebaseLine::Action { subject, .. } => {
            wrap_subject(subject, wrap_width).len()
        }
        RebaseLine::Comment(_) => 1,
    })
    .collect();

// Compute scroll offset: find the row offset (in terminal lines) such that
// the selected row is near the middle of the visible area.
let selected_line_height = selected_line_idx
    .map(|idx| row_heights[idx])
    .unwrap_or(0);

// Sum terminal lines up to selected row
let lines_before_selected: usize = selected_line_idx
    .map(|idx| row_heights[..idx].iter().sum())
    .unwrap_or(0);

// Determine scroll_offset_rows (how many ROWS to skip, not lines)
// Strategy: skip rows until lines_accumulated >= target_skip_lines
let target_skip_lines = lines_before_selected.saturating_sub(visible_height / 2);
let mut scroll_offset_rows = 0;
let mut accumulated = 0usize;
for (i, &h) in row_heights.iter().enumerate() {
    if accumulated >= target_skip_lines {
        scroll_offset_rows = i;
        break;
    }
    accumulated += h;
    scroll_offset_rows = i + 1;
}

// Then .skip(scroll_offset_rows).take(...) until terminal lines filled
```

**Simpler alternative for MVP:** Because the decision context says D-05 only requires "accounting" for wrapping, a simpler approach is to pass `wrap_width` to the scroll calculation and compute how many terminal lines each row takes, then clamp to fit `visible_height` lines total.

### Pattern 3: Wrapping at Word Boundaries (optional polish, Claude's Discretion)

The `wrap_subject` helper above breaks at character count. A word-boundary break is more readable:

```rust
fn wrap_subject_words(subject: &str, wrap_width: usize) -> Vec<String> {
    // Prefer breaking at last space before wrap_width
    // Fall back to hard break if no space found
    ...
}
```

This is low complexity and makes the output match user expectations. Recommended as default behavior.

### Anti-Patterns to Avoid
- **Setting `Cell::from(long_string)` without `Row::height(n)`:** ratatui silently truncates the cell to the first line. The `height` must match the wrapped line count or extra lines are invisible.
- **Using `area.width - 16` without `.saturating_sub()`:** If terminal is very narrow, subtraction panics on `u16` overflow. Always use `saturating_sub(16)`.
- **Reusing scroll_offset row count directly:** `skip(scroll_offset)` on rows still works, but `take(visible_height)` must become `take_while lines_used < visible_height` to avoid overflowing the terminal area with tall rows.
- **Forgetting comment row colspan:** Comment rows use only one `Cell` that spans the full width. Wrapping logic must not apply the 16-char indent to comment rows.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Multi-line cell display | Custom drawing loop with `buf.set_string` | `Cell::from(Text)` + `Row::height(n)` | ratatui Table handles clipping, style inheritance, column alignment automatically |
| Unicode-safe char slicing | Byte-indexed substring | `str::char_indices()` or `chars().take()` | Rust strings are UTF-8; byte slicing at non-char boundaries panics |
| Word wrap algorithm | Custom tokenizer | Simple last-space scan (5-10 lines) | Commit subjects don't contain complex Unicode, no need for full unicode-linebreak crate |

**Key insight:** ratatui `Row::height(n)` + `Text` with newlines is the canonical multi-line row pattern. It is documented in the official ratatui Row source and in use in ratatui examples. No custom drawing needed.

## Common Pitfalls

### Pitfall 1: Row Height Not Set to Match Line Count
**What goes wrong:** Cell content is `"first line\nsecond line"` but `Row` uses default `height(1)` — only the first line renders, second is silently clipped.
**Why it happens:** ratatui `Row` defaults to height 1. `Row::height()` must be called explicitly.
**How to avoid:** Always compute `n_lines = wrapped.len()` and call `.height(n_lines as u16)`.
**Warning signs:** Long subjects appear truncated in the same way as before the fix.

### Pitfall 2: Scroll Offset Overruns Visible Area
**What goes wrong:** `take(visible_height)` counts rows, but multi-line rows each consume more than 1 terminal line — causing rows to render past the bottom of the area and overlay the status bar.
**Why it happens:** The scroll loop was written assuming 1 row = 1 terminal line.
**How to avoid:** Replace `take(visible_height)` with a `take_while` accumulator that stops when accumulated terminal lines would exceed `visible_height`.
**Warning signs:** Status bar text is partially overwritten by rebase table rows on long subjects.

### Pitfall 3: Wrap Width Using Frame Width Instead of Subject Column Width
**What goes wrong:** Subject text wraps too early (action/hash columns are already subtracted by ratatui layout, so using `frame.area().width` directly causes double-subtraction).
**Why it happens:** Misunderstanding of which width to pass: we compute pre-broken lines for the subject column, so the available width is `area.width - 16` (layout-allocated) not `area.width`.
**How to avoid:** Use `area.width.saturating_sub(16) as usize` as the wrap width. This matches D-04.
**Warning signs:** Subjects with ~80 char terminal widths wrap at 64 chars instead of 80.

### Pitfall 4: Continuation Line Indent Not Aligned
**What goes wrong:** Continuation lines start at column 0 or misalign with the subject column.
**Why it happens:** The Cell only receives the subject-column content. Indenting within the Cell is relative to the left edge of that column. The subject column starts at column 16 (after action + hash). A continuation line inside the Cell must be blank-padded to 0 (no indent needed within the cell).
**How to avoid:** Continuation lines within the subject Cell do NOT need leading spaces — the Cell starts at the correct column. Any leading spaces in the continuation line will push content further right, misaligning it.
**Warning signs:** Continuation lines appear indented past the subject column start.

### Pitfall 5: Comment Row with Single Cell — Width Mismatch
**What goes wrong:** Comment rows use `Row::new(vec![Cell::from(text)])` with a single cell, but the table's `widths` array has 3 constraints. Comment content may be clipped at 8 chars (first constraint width).
**Why it happens:** ratatui Table assigns cells to widths by position. A 1-cell row gets the first constraint (8 chars for action column), not the full width.
**How to avoid:** This is the existing behavior that already works (comments display in full gray). Confirm the existing comment Row rendering is correct before modifying. If it uses `Constraint::Length(8)` for column 0, comment text may already be truncated — but this is pre-existing and out of scope for this phase.
**Warning signs:** Comment lines are truncated at 8 chars after the refactor.

## Code Examples

Verified patterns from ratatui 0.29.0 source:

### Multi-line Row with Height
```rust
// Source: ratatui-0.29.0/src/widgets/table/row.rs lines 151-162
let cells = vec!["Cell 1\nline 2", "Cell 2", "Cell 3"];
let row = Row::new(cells).height(2);
// height(2) causes both lines to render; without it, "line 2" is clipped
```

### Text::raw with Newlines Splits into Lines
```rust
// Source: ratatui-0.29.0/src/text/text.rs lines 240-246
// Text::raw("foo\nbar") yields Text { lines: [Line::from("foo"), Line::from("bar")] }
// Cell::from("foo\nbar") works the same way via the From<T: Into<Text>> impl
Cell::from("first line\nsecond line")
// When Row::height(2) is set, both lines render correctly
```

### Char-Safe String Slicing (for wrap helper)
```rust
// Source: Rust std — char_indices() gives (byte_offset, char) pairs
let chars: Vec<char> = subject.chars().collect();
let chunk: String = chars[..take_n].iter().collect();
// Matches existing Renderer::scroll_line() pattern in src/renderer.rs:153-162
```

### Wrap Width Calculation
```rust
// area is Rect passed to render_rebase_table()
// Constraint::Length(8) x2 = 16 chars for action + hash columns (D-04)
let wrap_width = area.width.saturating_sub(16) as usize;
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `Row::height(1)` default, subject truncated | `Row::height(n)` + pre-broken subject Text | This phase | Long subjects visible |
| Scroll counts rows | Scroll counts terminal lines | This phase | No status bar overlap with tall rows |

## Open Questions

1. **Word-boundary vs character-boundary wrapping**
   - What we know: Character-boundary is simpler and correct. Word-boundary is more readable.
   - What's unclear: Whether git commit subjects ever have very long words (e.g., URLs) that make char-boundary preferable anyway.
   - Recommendation: Implement word-boundary with char-boundary fallback — 10-15 lines of Rust, low risk.

2. **Continuation line indent within subject Cell**
   - What we know: The subject `Cell` starts at column 16. A continuation line within the Cell at position 0 aligns with the subject column start.
   - What's unclear: Whether the user expects indentation within the continuation line (e.g., 2-space visual indent) for readability.
   - Recommendation: Start with 0-indent within the Cell (aligns cleanly). The CONTEXT.md example shows alignment under subject start, which is what 0-indent within the Cell achieves.

3. **Selected row highlight spanning wrapped lines**
   - What we know: `Row::style(row_style)` applies the background/bold to the entire row height.
   - What's unclear: Whether the highlight color fills all terminal lines of the row including continuation lines.
   - Recommendation: Test with a selected long-subject row. ratatui Row style applies uniformly to all cells and all height lines — this should work correctly.

## Environment Availability

Step 2.6: SKIPPED (no external dependencies — pure Rust code change, no new tools or services required)

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test runner (cargo test) |
| Config file | none (inline `#[cfg(test)]` modules) |
| Quick run command | `cargo test wrap_subject` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

This phase has no formal REQ IDs. Behaviors map to tests as follows:

| Behavior | Test Type | Automated Command | File Exists? |
|----------|-----------|-------------------|-------------|
| Subject shorter than wrap_width: no wrapping | unit | `cargo test wrap_subject_short` | Wave 0 |
| Subject equal to wrap_width: no wrapping | unit | `cargo test wrap_subject_exact` | Wave 0 |
| Subject longer than wrap_width: splits into 2+ lines | unit | `cargo test wrap_subject_long` | Wave 0 |
| Continuation lines: no leading indent in subject Cell | unit | `cargo test wrap_subject_indent` | Wave 0 |
| Unicode subject: char-safe split (no panic) | unit | `cargo test wrap_subject_unicode` | Wave 0 |
| Scroll accounting: tall rows don't overflow visible_height | unit | `cargo test scroll_lines_sum` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test wrap_subject`
- **Per wave merge:** `cargo test`
- **Phase gate:** `cargo test` green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] Unit tests for `wrap_subject()` helper — to be added inline in `renderer.rs` `#[cfg(test)]` module
- [ ] Unit test for scroll line-height accumulation logic

*(Existing test infrastructure in `renderer.rs`, `document.rs`, `app.rs` requires no changes — new tests are additive)*

## Sources

### Primary (HIGH confidence)
- `ratatui-0.29.0` local cargo registry source — `src/widgets/table/row.rs`, `src/widgets/table/cell.rs`, `src/text/text.rs` — Row::height(), Cell::from(Text), Text::raw() newline splitting
- `src/renderer.rs` (project source) — render_rebase_table() lines 314-378, scroll_line() lines 153-162, area.width pattern lines 55-56
- `src/app.rs` (project source) — rebase_lines(), selected_rebase_line_idx(), scroll state architecture

### Secondary (MEDIUM confidence)
- CONTEXT.md Phase 6 — locked decisions D-01 through D-05, wrap_width = terminal_width - 16
- STATE.md — ratatui 0.30 confirmed via `cargo metadata`, stack decisions

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- ratatui Row/Cell multi-line API: HIGH — verified from local registry source of ratatui-0.29.0 (same API in 0.30)
- Wrap width calculation: HIGH — derived directly from locked decision D-04 and confirmed against existing constraint definitions in renderer.rs
- Scroll offset refactor: HIGH — current algorithm is simple and well-understood; refactoring approach is straightforward
- Pitfalls: HIGH — derived from direct source inspection, not external claims

**Research date:** 2026-03-30
**Valid until:** 2026-06-30 (stable ratatui API, unlikely to change)
