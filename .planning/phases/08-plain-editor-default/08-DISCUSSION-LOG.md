# Phase 8: Plain Editor Default - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-12
**Phase:** 08-plain-editor-default
**Areas discussed:** Squash header scope, Merge ConflictMarkers, Subject-line counter

---

## Squash Header Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Stay protected | Header block stays filtered via EditorMode; user edits only combined message below | ✓ |
| Fully editable | All lines including auto-header go into textarea | |

**User's choice:** Stay protected

---

| Option | Description | Selected |
|--------|-------------|----------|
| Editable (below header) | Only header block protected; trailing status `#` comments become editable | |
| All comments protected | Any `#` line in squash mode stays out of textarea | ✓ |

**User's choice:** All comments protected — squash mode is fully unchanged from current behavior.

**Notes:** This means Phase 8 has zero behavioral change in squash mode. The EditorMode flag exists to gate commit mode specifically.

---

## Merge ConflictMarkers

| Option | Description | Selected |
|--------|-------------|----------|
| Stay protected | ConflictMarker lines stay filtered; only `#` comments become editable in merge | |
| Editable too | ConflictMarker lines join textarea and are editable | ✓ |

**User's choice:** Editable — ConflictMarker lines treated as Content.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Editable (comments) | `#` comment lines in merge mode go into textarea | |
| Stay protected | `#` comment lines in merge mode stay out of textarea | ✓ |

**User's choice:** `#` comments in merge mode stay protected. Only ConflictMarker classification changes.

**Follow-up clarification asked:** Intent for ConflictMarker editability — "all non-comment content editable" vs "just conflict resolution flow."
**Answer:** All non-comment content editable — ConflictMarker lines are structural text the user might want to modify or remove.

---

## Subject-Line Counter

| Option | Description | Selected |
|--------|-------------|----------|
| First non-comment line | Counter skips `#` lines and anchors to first non-comment textarea line | |
| First textarea line always | Counter applies to line 1 regardless | |
| Remove counter in plain mode | No subject-line counter when EditorMode is plain | ✓ |

**User's choice:** Remove counter in plain mode.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Commit mode only | Counter removed only for commit EditorMode | |
| All modes | Counter removed everywhere | ✓ |

**User's choice:** All modes — remove the 50-char subject-line counter and 72-char body warning across commit, merge, and squash modes.

---

## Claude's Discretion

- `EditorMode` enum naming and variant structure
- Whether ConflictMarker promotion uses mode-aware `classify_line()` or extended `editable_index` logic
- Test update strategy (update existing tests vs add new ones)

## Deferred Ideas

None.
