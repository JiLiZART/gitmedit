---
phase: "06-rebase-view-horizontal-scrolling"
plan: "01"
subsystem: "renderer"
tags: ["rust", "ratatui", "word-wrap", "rebase", "tui"]
dependency_graph:
  requires: []
  provides: ["wrap_subject helper", "multi-line rebase rows", "terminal-line-aware scroll"]
  affects: ["src/renderer.rs", "render_rebase_table()"]
tech_stack:
  added: []
  patterns: ["word-boundary wrapping with char-boundary fallback", "pre-computed row_heights for variable-height scroll centering"]
key_files:
  created: []
  modified:
    - path: "src/renderer.rs"
      role: "Added wrap_subject() helper and rewrote render_rebase_table() with multi-line rows and terminal-line-aware scroll"
decisions:
  - "wrap_subject is a standalone fn (not impl Renderer method) to match counter_color_for / blank_warning_span pattern"
  - "wrap_width = area.width.saturating_sub(16): 8 chars for action column + 8 chars for hash column"
  - "Continuation lines are not indented (Cell column position handles alignment per RESEARCH Pitfall 4)"
  - "Scroll uses terminal-line-aware centering: row_heights pre-computed, skip rows until target_skip terminal lines consumed"
  - "Row loop breaks when lines_used + h > visible_height (not exceeding visible area except for first row which always renders)"
metrics:
  duration_min: 2
  completed_date: "2026-04-02"
  tasks_completed: 2
  files_modified: 1
---

# Phase 06 Plan 01: Word-wrap Long Rebase Subjects Summary

Word-wrapping for long rebase commit subjects using `wrap_subject()` helper and terminal-line-aware scroll offset centering in `render_rebase_table()`.

## What Was Built

### Task 1: wrap_subject() Helper (TDD)

Added a standalone `fn wrap_subject(subject: &str, wrap_width: usize) -> Vec<String>` function in `src/renderer.rs` above the existing `counter_color_for` helper.

Implementation strategy:
- Zero width or subject fits: return single-element vec immediately
- Scan backwards from `start + wrap_width` to find last space for word-boundary break
- If no space found in range: hard-break at exactly `wrap_width` chars
- Continuation lines trim leading whitespace (trim from remainder after space)
- Unicode-safe: operates on `chars()` collected to `Vec<char>`, never slices bytes

8 unit tests added to the existing `#[cfg(test)] mod tests` block:
- `test_wrap_subject_short_no_wrap`
- `test_wrap_subject_exact_fit`
- `test_wrap_subject_wraps_at_word_boundary`
- `test_wrap_subject_hard_break_no_spaces`
- `test_wrap_subject_empty`
- `test_wrap_subject_zero_width`
- `test_wrap_subject_unicode_safe`
- `test_wrap_subject_no_continuation_indent`

### Task 2: Integration into render_rebase_table()

Rewrote `render_rebase_table()` with three key changes:

**1. wrap_width computation:**
```rust
let wrap_width = area.width.saturating_sub(16) as usize;
```

**2. Pre-computed row_heights for scroll accounting:**
```rust
let row_heights: Vec<usize> = rebase_lines.iter().map(|line| match line {
    RebaseLine::Action { subject, .. } => wrap_subject(subject, wrap_width).len(),
    RebaseLine::Comment(_) => 1,
}).collect();
```

**3. Terminal-line-aware scroll centering** replaces the former `idx.saturating_sub(visible_height/2)` row-count approach. The new algorithm sums terminal lines before the selected row, targets `lines_before - visible_height/2`, and walks row_heights to find the correct skip boundary.

**4. Manual row loop** with `lines_used` tracking replaces `.take(visible_height)`. Breaks when `lines_used + h > visible_height` (with guard: first row always renders).

**5. Multi-line Cell content:**
```rust
let subject_text = wrapped_lines.join("\n");
Row::new(vec![...Cell::from(subject_text)...]).height(height)
```

## Decisions Made

| Decision | Rationale |
|----------|-----------|
| wrap_subject as standalone fn | Matches existing helper pattern (counter_color_for, blank_warning_span) |
| wrap_width = area.width - 16 | action=8 + hash=8, subject column starts at col 16 |
| No indent on continuation lines | Cell is positioned at column 16; indent within Cell would add extra offset |
| Pre-compute row_heights Vec | Needed for both scroll offset and row-loop termination; single pass |
| Manual loop over .take_while | Cleaner than closure-based state mutation for lines_used tracking |

## Test Results

- `cargo test`: 94 tests pass (8 new wrap_subject tests + 86 existing)
- `cargo build`: clean, no errors (pre-existing dead_code warnings in document.rs are unrelated to this plan)

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. All subjects are wired to live `RebaseLine::Action { subject }` data from the parsed rebase-todo file.

## Self-Check: PASSED

- [x] src/renderer.rs modified (contains wrap_subject, render_rebase_table updates)
- [x] Task 1 commit f47e964 exists
- [x] Task 2 commit bb21cad exists
- [x] All 94 cargo tests pass
- [x] cargo build exits 0 with no errors
