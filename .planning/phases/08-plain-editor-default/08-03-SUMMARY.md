---
phase: 08-plain-editor-default
plan: "03"
subsystem: app
tags: [rust, ratatui, editor, app-wiring]
dependency_graph:
  requires: [08-01]
  provides: [commit-mode comment editability, merge-mode conflict marker promotion, app-level tests]
  affects: [src/app.rs]
tech_stack:
  added: []
  patterns: [EditorMode match in App::new, Document::parse 3-arg call]
key_files:
  created: []
  modified:
    - src/app.rs
decisions:
  - "EditorMode wiring was already applied by Plan 01 Rule 3 auto-fix; Plan 03 adds the 4 required inline tests"
  - "Tests verify textarea contents directly via app.textarea().lines() and Document::serialize independently"
metrics:
  duration: "5 minutes"
  completed: "2026-05-14"
  tasks_completed: 1
  files_modified: 1
---

# Phase 08 Plan 03: EditorMode App::new Wiring + Tests Summary

**One-liner:** App::new selects EditorMode from GitContext and passes it to Document::parse; 4 new inline tests prove commit-mode comment editability and merge-mode conflict-marker promotion.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Add EditorMode selection + 4 inline tests | 992dfa2 | src/app.rs |

## What Was Done

### EditorMode Wiring (already present from Plan 01 Rule 3 auto-fix)

The EditorMode match block and Document::parse 3-arg call were already in `src/app.rs`:

```rust
let editor_mode = match context {
    GitContext::Commit => EditorMode::Plain,
    GitContext::Merge  => EditorMode::Merge,
    GitContext::Squash => EditorMode::Squash,
    _                  => EditorMode::Plain,
};
let document = Document::parse(&content_for_document, comment_char, editor_mode);
```

### 4 New Tests Added

| Test | Mode | Verifies |
|------|------|---------|
| `test_app_commit_mode_includes_comment_lines_in_textarea` | Commit/Plain | `# comment` appears in textarea lines |
| `test_app_commit_mode_serializes_back_to_original` | Commit/Plain | Round-trip: comment line preserved verbatim |
| `test_app_commit_mode_edited_comment_persists` | Plain (Document direct) | Edited comment line written through serialize |
| `test_app_merge_mode_promotes_conflict_markers` | Merge | `<<<<<<< HEAD`, `=======`, `>>>>>>> br` in textarea; `# msg` excluded |

## Test Results

- Before plan: 92 tests passed, 1 ignored
- After plan: **96 tests passed, 1 ignored** (+4 new)
- Squash regression checks: `test_app_squash_serialized_content` PASS, `test_app_squash_no_header_falls_back_to_normal` PASS

## Phase 8 Success Criteria — End-to-End Status

| Criterion | Status |
|-----------|--------|
| Commit mode: cursor can move over and overwrite `#` comment lines | DONE — `EditorMode::Plain` includes Comment in editable_index |
| Default ratatui_textarea behavior applies to every line in textarea | DONE — Plain mode passes all lines to textarea unchanged |
| Squash mode behavior unchanged | DONE — `EditorMode::Squash` still protects Comment/ConflictMarker |
| Merge mode: ConflictMarker promoted, Comment protected | DONE — `EditorMode::Merge` tested in `test_app_merge_mode_promotes_conflict_markers` |

## External Callers of App::new

`src/main.rs` calls `App::new(content, context)` — no changes needed since `GitContext` is already its argument and the mode selection is entirely internal to `App::new`.

## Deviations from Plan

### Context-Driven Scope Reduction

**[Plan 01 Rule 3 Auto-fix already covered Step 1-3]**
- **Found during:** Reading 08-01-SUMMARY.md before starting
- **Issue:** Plan 01 updated `app.rs` caller as a Rule 3 auto-fix to unblock compilation; the EditorMode import, match block, and Document::parse 3-arg call were all already present
- **Scope:** Plan 03's unique contribution is the 4 new inline tests (Steps 5-6 of the original action plan)
- **Files modified:** `src/app.rs` (tests section only)
- **Commit:** 992dfa2

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Tests only. Consistent with threat model T-08-04 (accepted — EditorMode selected from internal GitContext, not file content).

## Self-Check: PASSED

- `src/app.rs` modified: FOUND
- Commit 992dfa2: FOUND
- `let editor_mode = match context` in app.rs: FOUND (line 78)
- `Document::parse(&content_for_document, comment_char, editor_mode)` in app.rs: FOUND (line 84)
- 4 new test functions: FOUND (lines 435, 441, 447, 462)
- `cargo test` — 96 passed, 0 failed
