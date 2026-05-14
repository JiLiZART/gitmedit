---
phase: 08-plain-editor-default
plan: "02"
subsystem: renderer
tags: [rust, ratatui, editor, ui, cleanup]
dependency_graph:
  requires: []
  provides: [counter-free-status-bars]
  affects: [src/renderer.rs]
tech_stack:
  added: []
  patterns: [single-span-status-bar]
key_files:
  created: []
  modified:
    - src/renderer.rs
decisions:
  - "Kept `let _ = app;` in both status bar functions to preserve stable signatures for Phase 9 command-bar additions"
  - "Modifier import retained — still used by rebase status bar bold spans"
metrics:
  duration: "5 minutes"
  completed: "2026-05-14"
  tasks_completed: 1
  tasks_total: 1
requirements:
  - EDIT-11
---

# Phase 08 Plan 02: Remove Subject Counter and Blank-Line Warning Summary

Deleted the 50-char subject-line counter and "[No blank line]" warning from both commit and squash status bars. Both bars now render a single plain actions span.

## Final Status Bar Bodies

**render_status_bar:**
```rust
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let _ = app;
    let actions_span = Span::raw("^S Save  Esc Cancel  ^H Help");
    let status_line = Line::from(vec![actions_span]);
    let status_widget = Paragraph::new(status_line)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));
    frame.render_widget(status_widget, area);
}
```

**render_squash_status_bar:**
```rust
fn render_squash_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let _ = app;
    let actions_span = Span::raw("^S Save  Esc Cancel  ^H Help  [Squash]");
    let status_line = Line::from(vec![actions_span]);
    let status_widget = Paragraph::new(status_line)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));
    frame.render_widget(status_widget, area);
}
```

## Deleted Symbols

- `counter_color_for(count: usize) -> Color` — color thresholds at 50/72 chars
- `blank_warning_span(has_blank: bool) -> Span<'static>` — "[No blank line]" warning span
- 9 unit tests: `test_counter_color_green_at_50`, `test_counter_color_yellow_at_60`, `test_counter_color_red_at_80`, `test_counter_color_green_at_0`, `test_counter_color_yellow_at_51`, `test_counter_color_yellow_at_72`, `test_counter_color_red_at_73`, `test_blank_line_warning_not_shown_when_present`, `test_blank_line_warning_shown_when_missing`
- Two divider comment blocks (`// counter_color_for`, `// blank_warning_span`)

## Unused Imports Removed

None — `Modifier` remains used by the rebase status bar's bold key hints.

## Document Method Dependencies

`Document::first_line()` and `Document::has_blank_line_after_subject()` are no longer called from `renderer.rs`. They remain on `Document` for use by document tests and potential future use. Plan 03 may leave them in place.

## Deviations from Plan

None — plan executed exactly as written.

## Threat Flags

None — this plan only deleted UI rendering code; no new inputs, I/O, or network access introduced.

## Self-Check: PASSED

- `src/renderer.rs` modified: confirmed
- Commit `2c8ef7c` exists: confirmed
- `grep -c counter_color_for src/renderer.rs` == 0: confirmed
- `grep -c blank_warning_span src/renderer.rs` == 0: confirmed
- `cargo build` exits 0: confirmed
- `cargo test renderer` 8 passed: confirmed
