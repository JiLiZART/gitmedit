---
phase: 08-plain-editor-default
plan: "01"
subsystem: document
tags: [rust, ratatui, editor, parse-serialize]
dependency_graph:
  requires: []
  provides: [EditorMode enum, mode-aware Document::parse, mode-aware editable_lines, mode-aware serialize]
  affects: [src/document.rs, src/app.rs]
tech_stack:
  added: []
  patterns: [EditorMode enum flag on Document, editable_index-driven editable_lines, matches!() mode branches]
key_files:
  created: []
  modified:
    - src/document.rs
    - src/app.rs
decisions:
  - "EditorMode variants named Plain/Merge/Squash (matches existing GitContext naming)"
  - "editable_lines drives off editable_index rather than re-filtering lines by variant"
  - "serialize Plain short-circuit returns textarea_lines.join + trailing newline"
  - "App::new updated in same plan (Rule 3: blocking compile error) — caller wiring plan-03 handles further changes"
metrics:
  duration: "12 minutes"
  completed: "2026-05-14"
  tasks_completed: 2
  files_modified: 2
---

# Phase 08 Plan 01: EditorMode enum + mode-aware parse/editable_lines/serialize Summary

**One-liner:** Mode-gated EditorMode enum on Document drives plain-editor behavior: Comment editable in Plain, ConflictMarker editable in Plain+Merge, both protected in Squash.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Add EditorMode enum and thread through parse | 22c0053 | src/document.rs, src/app.rs |
| 2 | Plain-mode serialize short-circuit + roundtrip tests | 7314ba6 | src/document.rs |

## EditorMode Variants

| Variant | Semantics |
|---------|-----------|
| `Plain` | All lines (Content + Comment + ConflictMarker) are editable; serialize emits textarea verbatim |
| `Merge` | Content + ConflictMarker are editable; Comment is protected and re-injected verbatim |
| `Squash` | Only Content is editable; Comment and ConflictMarker are protected (unchanged from pre-plan behavior) |

## Document Changes

### New: `EditorMode` enum
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorMode {
    Plain,
    Merge,
    Squash,
}
```

### Document struct: new `mode: EditorMode` field + `pub fn mode(&self) -> EditorMode` accessor

### Updated: `Document::parse` signature
```rust
pub fn parse(raw: &str, comment_char: char, mode: EditorMode) -> Document
```
Mode-aware editable_index gate:
```rust
let include = match &classified {
    ContentLine::Content(_) => true,
    ContentLine::Comment(_) => matches!(mode, EditorMode::Plain),
    ContentLine::ConflictMarker(_) => matches!(mode, EditorMode::Plain | EditorMode::Merge),
};
```

### Updated: `editable_lines()` — drives off editable_index
```rust
pub fn editable_lines(&self) -> Vec<String> {
    self.editable_index.iter().map(|&i| match &self.lines[i] {
        ContentLine::Content(s) | ContentLine::Comment(s) | ContentLine::ConflictMarker(s) => s.clone(),
    }).collect()
}
```

### Updated: `Document::serialize` — Plain short-circuit + Merge/Squash split
- Plain: `textarea_lines.join("\n") + "\n"` returned immediately
- Merge: Comment re-injected verbatim; ConflictMarker drawn from textarea
- Squash: both Comment and ConflictMarker re-injected verbatim (unchanged)

## Tests Added and Updated

### New Tests (7 total)
| Test | Mode | Verifies |
|------|------|---------|
| `test_editor_mode_plain_makes_comments_editable` | Plain | Comments appear in editable_lines |
| `test_editor_mode_plain_makes_conflict_markers_editable` | Plain | Markers appear in editable_lines |
| `test_editor_mode_merge_protects_comments_but_promotes_conflict_markers` | Merge | Comments excluded; markers included |
| `test_editor_mode_squash_unchanged` | Squash | Only Content in editable_lines |
| `test_serialize_plain_mode_emits_textarea_verbatim` | Plain | Roundtrip with comment preserved |
| `test_serialize_plain_mode_edits_comment_line` | Plain | Edited comment line written through |
| `test_serialize_merge_mode_preserves_comment_promotes_marker` | Merge | Comment verbatim; marker from textarea |

### Updated Tests (14 existing tests received `EditorMode::Squash` arg)
All pre-existing `Document::parse(input, '#')` calls updated to `Document::parse(input, '#', EditorMode::Squash)` to preserve legacy assertions.

**Total passing: 53 document tests (was 46 pre-plan)**

## App.rs Changes

`App::new` updated to import `EditorMode` and select mode from `GitContext`:
```rust
let editor_mode = match context {
    GitContext::Commit => EditorMode::Plain,
    GitContext::Merge  => EditorMode::Merge,
    GitContext::Squash => EditorMode::Squash,
    _                  => EditorMode::Plain,
};
let document = Document::parse(&content_for_document, comment_char, editor_mode);
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated app.rs caller in same plan**
- **Found during:** Task 1 (compile check)
- **Issue:** `Document::parse` signature changed from 2 to 3 args; `app.rs` call site would not compile
- **Fix:** Updated `app.rs` import to include `EditorMode`, added `editor_mode` selection logic before `Document::parse` call — matches the pattern documented in `08-PATTERNS.md` (plan-03 template)
- **Files modified:** `src/app.rs`
- **Commit:** 22c0053

Plan-03 (caller wiring) is still needed for any further App::new wiring; this fix only makes the existing call site compile with the new 3-arg signature.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Serialize emits to local tempfile processed by git — consistent with existing threat model T-08-01 (accepted).

## Self-Check: PASSED

- `src/document.rs` modified: FOUND
- `src/app.rs` modified: FOUND
- Commit 22c0053: FOUND (git log)
- Commit 7314ba6: FOUND (git log)
- `cargo test document::` — 53 passed, 0 failed
