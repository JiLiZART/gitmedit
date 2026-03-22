---
phase: 03-commit-message-intelligence
plan: 01
subsystem: document-renderer
tags: [commit-intelligence, status-bar, character-counter, blank-line-detection]
dependency_graph:
  requires: []
  provides: [first_line, has_blank_line_after_subject, counter_color_for, blank_warning_span, render_status_bar_with_counter]
  affects: [src/document.rs, src/renderer.rs]
tech_stack:
  added: []
  patterns: [pub(crate) helper extraction for testability, chars().count() for Unicode safety, trim().is_empty() for blank line detection]
key_files:
  created: []
  modified:
    - src/document.rs
    - src/renderer.rs
decisions:
  - "Extract counter_color_for() and blank_warning_span() as pub(crate) helpers — makes status bar color/warning logic independently unit-testable without full frame rendering"
  - "first_line() uses editable_lines().first().cloned().unwrap_or_default() — consistent with existing editable_lines() pattern and safe for empty documents"
  - "has_blank_line_after_subject() returns true for single-line messages — single-line commits don't require a blank separator"
metrics:
  duration_minutes: 3
  completed_date: "2026-03-22"
  tasks_completed: 3
  files_modified: 2
---

# Phase 3 Plan 1: Subject Line Counter and Blank Line Detection Summary

Real-time subject line character counter (green/yellow/red at 50/72 thresholds) and blank line separator detection displayed in the status bar, backed by Document methods and unit-tested helper functions.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add first_line() and has_blank_line_after_subject() to Document | ba91373 | src/document.rs |
| 2 | Update render_status_bar() with char counter and blank line warning | 51dc2df | src/renderer.rs |
| 3 | Add unit tests for counter colors and blank line rendering | 9bd497c | src/renderer.rs |

## What Was Built

**Document methods (src/document.rs):**
- `first_line() -> String` — returns first editable (Content) line; empty string if no editable lines
- `has_blank_line_after_subject() -> bool` — detects blank separator using `trim().is_empty()`; single-line messages return true

**Renderer (src/renderer.rs):**
- `render_status_bar()` updated to display `"Chars: {n}"` in bold green/yellow/red
- Blank line warning `" [No blank line]"` in yellow when separator is missing
- Actions text updated to `"^S Save  Esc Cancel  ^H Help"`
- `counter_color_for(count: usize) -> Color` — pub(crate) helper, testable in isolation
- `blank_warning_span(has_blank: bool) -> Span` — pub(crate) helper, testable in isolation

## Test Coverage

- 5 new unit tests in `document.rs`: subject extraction, empty document, blank line variants
- 9 new unit tests in `renderer.rs`: color boundaries (0, 50, 51, 60, 72, 73, 80), warning presence/absence
- Total: 47 tests passing (was 38 before this plan)

## Deviations from Plan

**1. [Rule 2 - Missing Critical Functionality] Extracted helpers for testability**
- **Found during:** Task 3
- **Issue:** `render_status_bar()` is a private method taking `Frame` — cannot be tested without terminal setup. Plan mentioned "extract logic into helper function for easier testing" as an option.
- **Fix:** Extracted `counter_color_for()` and `blank_warning_span()` as `pub(crate)` functions outside `impl Renderer`. Status bar delegates to these helpers. Plan explicitly sanctioned this approach.
- **Files modified:** src/renderer.rs
- **Commit:** 9bd497c

## Success Criteria Verification

- [x] COMMIT-01: Subject line character counter displays in real-time (status bar updates on every keystroke via normal render loop)
- [x] COMMIT-02: Counter shows correct colors (green <=50, yellow 51-72, red >72) — verified via unit tests and boundary tests
- [x] COMMIT-03: Blank line enforcement indicator appears/disappears correctly — verified via unit tests
- [x] Character count uses `.chars().count()` for Unicode safety
- [x] Blank line detection uses `.trim().is_empty()` to ignore trailing whitespace
- [x] All unit tests pass: 47/47

## Self-Check: PASSED
