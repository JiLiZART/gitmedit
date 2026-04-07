# Phase 03: Commit Message Intelligence - Validation Strategy

**Created:** 2026-03-22
**Source:** Research (03-RESEARCH.md)

<validation_scope>
## Validation Scope

Map each requirement to test type (unit/integration/manual):

| Requirement | Behavior | Test Type | Coverage |
|-------------|----------|-----------|----------|
| COMMIT-01 | Subject line counter displays in real-time | unit + manual | Task 01-1, Task 01-2, Task 01-3 |
| COMMIT-02 | Color coding applied (green ≤50, yellow 51-72, red >72) | unit | Task 01-3 (test_counter_color_*) |
| COMMIT-03 | Blank line detection between subject and body | unit | Task 01-1 (test_has_blank_line_*) |
| COMMIT-06 | Hotkey help triggered by Ctrl+H | integration | Task 02-3 (event loop gating) |
| HELP-01 | Hotkey reference available (Ctrl+H) | integration | Task 02-3 (Ctrl+H binding) |
| HELP-02 | Help shows only relevant actions for current mode | unit | Task 02-2 (test_help_text_for_context) |
| HELP-03 | Help does not interfere with editing | integration | Task 02-3 (input gating) |
| HELP-04 | Help can be dismissed with Esc, cursor preserved | integration + manual | Task 02-2, Task 02-3 (modal rendering, state preservation) |

</validation_scope>

<dimension_map>
## Dimension Map

### Dimension 1: Requirement Traceability

**All 8 requirements addressed:**

| Requirement ID | Plan | Task | Artifact |
|----------------|------|------|----------|
| COMMIT-01 | 03-01 | Task 1, Task 2 | src/document.rs:first_line(), src/renderer.rs status bar |
| COMMIT-02 | 03-01 | Task 2, Task 3 | src/renderer.rs color logic, test_counter_color_* tests |
| COMMIT-03 | 03-01 | Task 1, Task 2 | src/document.rs:has_blank_line_after_subject(), renderer warning display |
| COMMIT-06 | 03-02 | Task 3 | src/main.rs Ctrl+H hotkey binding |
| HELP-01 | 03-02 | Task 3 | src/main.rs Ctrl+H event handler |
| HELP-02 | 03-02 | Task 2 | src/renderer.rs:help_text_for_context() method |
| HELP-03 | 03-02 | Task 3 | src/main.rs input gating logic (if app.is_help_visible()) |
| HELP-04 | 03-02 | Task 2, Task 3 | src/renderer.rs modal rendering, no state mutation while visible |

### Dimension 2: Implementation Coverage

**Plan 03-01 — Subject Line Counter & Blank Line Detection:**
- src/document.rs: first_line() method (extract subject line)
- src/document.rs: has_blank_line_after_subject() method (detect blank line)
- src/renderer.rs: Updated render_status_bar() (display counter, warning)
- src/renderer.rs: Color logic (green/yellow/red thresholds)

**Plan 03-02 — Help Overlay:**
- src/app.rs: Action enum (Help, DismissHelp variants)
- src/app.rs: App struct (show_help boolean field, is_help_visible() getter)
- src/renderer.rs: centered_rect() helper (modal positioning)
- src/renderer.rs: help_text_for_context() method (mode-aware text)
- src/renderer.rs: render_help_overlay() method (modal rendering)
- src/main.rs: Ctrl+H hotkey binding (event loop integration)
- src/main.rs: Input gating logic (block editing while help visible)

### Dimension 3: Test Coverage Targets

**From Research §Validation Architecture:**

Unit tests:
- `src/document.rs` — test_first_line_returns_subject, test_first_line_empty_when_no_editable, test_has_blank_line_true_when_blank_present, test_has_blank_line_false_when_missing, test_has_blank_line_true_single_line
- `src/renderer.rs` — test_counter_color_green_at_50, test_counter_color_yellow_at_60, test_counter_color_red_at_80, test_blank_line_warning_not_shown_when_present, test_blank_line_warning_shown_when_missing, test_help_text_for_context (mode-aware variations)

Integration tests:
- `cargo test --lib` coverage: modal rendering with no alternate screen buffer
- `cargo test --lib` coverage: app state transitions (Help → DismissHelp)
- `cargo test --lib` coverage: input gating prevents textarea.input() while help visible

### Dimension 4: Manual Verification Checkpoints

- Manual: Counter updates smoothly on keystroke (real-time, no lag)
- Manual: Blank line warning appears/disappears correctly on edit
- Manual: Help modal centered, not truncated, readable
- Manual: Cursor position preserved after help dismiss
- Manual: Help text changes per git context (Commit vs Merge)

### Dimension 5: Regression Testing

- Run full suite: `cargo test --lib`
- Verify no changes to Phase 2 behavior (textarea, document parsing, save/cancel flow)
- Existing tests must still pass (test_first_line_returns_subject, etc., don't break existing tests)

</dimension_map>

<sampling_strategy>
## Nyquist Sampling Strategy

**Critical failure modes to monitor:**

### Mode 1: Character Counter
- **Failure mode:** Counter updates with lag or freezes
- **Sample point:** Type rapidly (>5 chars/sec) and verify counter updates every frame
- **Expected:** Smooth, real-time counter (no visible delay)

### Mode 2: Character Counter Edge Cases
- **Failure mode:** Counter at boundary (50 chars) doesn't transition color correctly
- **Sample points:** Exactly 50 chars (should be green), 51 chars (should be yellow), 72 chars (should be yellow), 73 chars (should be red)
- **Expected:** Color transitions at exact boundaries

### Mode 3: Blank Line Detection
- **Failure mode:** Comments or visual blank lines confuse detection; warning shows when it shouldn't
- **Sample points:**
  - Subject + actual blank line + body (should not warn)
  - Subject + comment line + body (should not warn — comments don't count)
  - Subject + body with no blank line (should warn)
  - Single-line message (should not warn)
- **Expected:** Warning appears only when no blank line in editable content

### Mode 4: Help Modal Rendering
- **Failure mode:** Modal overlaps incorrectly or clears content underneath
- **Sample point:** Open help, verify editor text still visible, dismiss help, verify text fully restored
- **Expected:** Modal centered, content visible underneath, no artifacts after dismiss

### Mode 5: Help Modal Dismissal
- **Failure mode:** Cursor or scroll position lost when help dismissed
- **Sample point:** Edit to line 5, col 10, open help, close help, verify cursor at line 5, col 10
- **Expected:** Exact position preservation

### Mode 6: Help Input Gating
- **Failure mode:** Keyboard input leaks through modal; typing while help visible modifies message
- **Sample point:** Open help, type "test", press Ctrl+S, verify no changes to message
- **Expected:** Help blocks all input except Esc/Ctrl+H

### Mode 7: Help Modal Context Awareness
- **Failure mode:** Help text same for Commit and Merge modes
- **Sample points:**
  - Commit mode: verify no conflict marker note
  - Merge mode: verify conflict marker note is shown
- **Expected:** Different help text per context

### Mode 8: Help Modal Toggle
- **Failure mode:** Ctrl+H doesn't toggle; stuck open
- **Sample point:** Press Ctrl+H to open, Ctrl+H again to close, verify closes
- **Expected:** Toggle behavior works in both directions

**Sampling frequency:**
- Per task commit: `cargo test --lib` (unit tests, <2s)
- Per wave merge: `cargo test` (full suite, <10s)
- Phase gate: Full suite + manual sampling on terminal (counter smoothness, modal rendering, input gating)

</sampling_strategy>

---

## Verification Commands

```bash
# Unit tests only (fast feedback per task)
cargo test --lib document::tests
cargo test --lib renderer::tests
cargo test --lib app::tests

# Full suite (after each wave)
cargo test --lib

# Manual verification (phase gate)
gitmedit /tmp/test_msg  # Type short, medium, long subject; add body with/without blank line
                        # Press Ctrl+H to open help, Esc to close, verify all behaviors

# Specific test runs by requirement
cargo test --lib document::tests::test_first_line                      # COMMIT-01
cargo test --lib renderer::tests::test_counter_color                   # COMMIT-02
cargo test --lib document::tests::test_has_blank_line                  # COMMIT-03
cargo test --lib app::tests::test_help_action                          # COMMIT-06, HELP-01
cargo test --lib renderer::tests::test_help_text_for_context           # HELP-02
cargo test --lib main integration tests (if available)                 # HELP-03, HELP-04
```

---

## Test File Locations

- `src/document.rs` — test module at bottom (5 tests for first_line, has_blank_line)
- `src/renderer.rs` — test module at bottom (5 tests for counter colors + warning, help text)
- `src/app.rs` — test module at bottom (tests for Help/DismissHelp actions, is_help_visible)
- `src/main.rs` — event loop integration (manual verification via gitmedit CLI)

---

## References

**Source:** 03-RESEARCH.md § Validation Architecture, § Common Pitfalls
**Confidence:** HIGH (verified against Phase 2 testing patterns)
**Maintained by:** Phase 3 executor
**Last updated:** 2026-03-22
