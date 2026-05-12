# Phase 8: Plain Editor Default - Context

**Gathered:** 2026-05-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Remove the comment-line read-only guard so that `#` comment lines (and, in merge mode, ConflictMarker lines) are editable by default using `ratatui_textarea`'s native behavior.

The change is **mode-gated via an `EditorMode` flag on Document** (pre-decided in STATE.md). Three modes are in scope:

- **Commit mode** — all lines (Content + Comment) go into textarea; fully editable
- **Merge mode** — ConflictMarker lines (`<<<<<<`, `=======`, `>>>>>>>`) become editable (treated as Content); `#` comment lines stay protected
- **Squash mode** — unchanged from today: auto-generated header block stays protected, AND all other `#` comment lines also stay protected; only Content lines are editable

Rebase mode is **out of scope** — it uses a separate table renderer, not textarea.

Subject-line counter (50-char warning) is **removed across all modes** as part of this phase.

</domain>

<decisions>
## Implementation Decisions

### Commit Mode
- **D-01:** All lines go into `textarea` — `ContentLine::Comment` lines are no longer filtered out. `editable_index` maps every line, not just `Content` lines, when `EditorMode::Plain`.
- **D-02:** `serialize()` must be updated for plain mode: since all lines are in textarea, there are no "Comment" lines to re-inject. The output is `textarea.lines().join("\n")` (plus trailing newline). The existing re-injection path still applies for squash/merge modes.

### Squash Mode
- **D-03:** Squash mode behavior is **fully unchanged**. The auto-generated header block (detected by `detect_squash_header()`) stays protected. All other `#` comment lines in squash also stay protected. `EditorMode` flag keeps squash on the existing code path.

### Merge Mode
- **D-04:** `ContentLine::ConflictMarker` lines (`<<<<<<< HEAD`, `=======`, `>>>>>>>`) become editable in merge mode — treated the same as `Content` for `editable_index` purposes.
- **D-05:** Regular `#` comment lines in merge mode stay protected (not in textarea). Merge mode `#` behavior = same as today. Only ConflictMarker classification changes.

### Subject-Line Counter
- **D-06:** Remove the 50-char subject-line counter and the 72-char body-line warning **across all modes** (commit, merge, squash). No per-mode exception.

### Claude's Discretion
- Exact `EditorMode` enum variants and naming (e.g., `EditorMode::Plain` vs `EditorMode::Commit`) — planner decides.
- How ConflictMarker editable promotion is expressed: whether `editable_index` logic is extended or whether `classify_line` is made mode-aware — planner decides.
- Whether existing `Document` tests for `editable_lines()` and `serialize()` need updating vs new tests added — planner decides test strategy.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §EDIT-10, EDIT-11 — definitions and traceability table
- `.planning/ROADMAP.md` §"Phase 8: Plain Editor Default" — success criteria (4 items)

### Core Files to Modify
- `src/document.rs` — `Document::parse()`, `editable_lines()`, `serialize()`, `classify_line()`, `detect_squash_header()`, `ContentLine` enum; this is the primary file for the EditorMode gate
- `src/app.rs` — `App::new()` (builds textarea from `editable_lines()`), `serialized_content()` (calls `serialize()`); needs mode-awareness passed through
- `src/document.rs:231` — `classify_line()` function — where ConflictMarker classification may become mode-aware

### Prior Decisions (carry-forward)
- `.planning/STATE.md` §Decisions — "[Pre-v1.1]: Plain editor via `EditorMode` flag on Document, not by removing Comment/ConflictMarker variants globally — prevents squash/merge corruption"

### Test Files to Update
- `src/app.rs` (inline tests) — `test_app_squash_serialized_content` (line ~404), squash header assertion; must still pass unchanged
- `src/document.rs` (inline tests) — editable_lines and serialize tests around Comment filtering (lines ~605–778)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `detect_squash_header(raw, comment_char)` in `src/document.rs:166` — already detects the squash header boundary; use it to gate squash protection in EditorMode
- `ContentLine::Comment(String)` and `ContentLine::ConflictMarker(String)` — variants to selectively include/exclude per mode
- `editable_index: Vec<usize>` in `Document` — the index that maps textarea rows to full line positions; EditorMode controls which line variants get pushed into it

### Established Patterns
- Mode-gating via enum flag on `Document` (pre-decided) — EditorMode flag travels with the Document from construction through to serialize
- `App::new()` builds textarea from `document.editable_lines()` — the entry point for the change; if EditorMode::Plain, editable_lines returns all lines
- `document.serialize(textarea.lines())` in `App::serialized_content()` — must handle the plain-mode case where all lines came from textarea (no Comment re-injection needed)

### Integration Points
- `App::new()` at `src/app.rs:80` — `TextArea::new(editable)` — receives the line slice; EditorMode determines what `editable` contains
- Subject-line counter render path — wherever the 50-char indicator is drawn, remove the counter call entirely (all modes)

</code_context>

<specifics>
## Specific Ideas

- "Plain editor" means the editor behaves like a dumb textarea: no special casing for `#` in commit mode. git itself strips comment lines on save anyway, so there's no corruption risk.
- Squash keeps its existing protection because the squash header carries semantic meaning that git reads.
- Merge ConflictMarker editability allows cursor to move over `<<<`/`===`/`>>>` lines naturally rather than jumping past them.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 8-plain-editor-default*
*Context gathered: 2026-05-12*
