# Phase 8: Plain Editor Default - Pattern Map

**Mapped:** 2026-05-14
**Files analyzed:** 2 (src/document.rs, src/app.rs)
**Analogs found:** 2 / 2 (self-referential — these ARE the files being modified)

---

## File Classification

| Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------|------|-----------|----------------|---------------|
| `src/document.rs` | model + transform | request-response (parse → edit → serialize) | `src/context.rs` (enum pattern) | role-match for enum; self-analog for parse/serialize |
| `src/app.rs` | controller / state machine | request-response | self (squash-mode branch) | exact — existing `if matches!(context, GitContext::Squash)` is the template |

---

## Pattern Assignments

### `src/document.rs` — `EditorMode` enum (new) + `Document` struct changes

**Pattern: Rust enum with `#[derive(Debug, Clone, Copy, PartialEq)]`**

Source: `src/context.rs` lines 4–11 (GitContext) and `src/document.rs` lines 1–9 (RebaseAction).

```rust
// Analog from src/context.rs:4-11
#[derive(Debug, Clone, PartialEq)]
pub enum GitContext {
    Commit,
    Merge,
    Rebase,
    Squash,
    Unknown,
}
```

New `EditorMode` enum must follow the same derive set. Use `Copy` too since it is a small flag (same as `RebaseAction`):

```rust
// Analog: src/document.rs:2-4
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RebaseAction { ... }
```

**Pattern: mode-gated struct field carried from construction**

Source: `src/app.rs` lines 29–38 — `App` holds `squash_log: Vec<String>` set only in squash mode; same pattern applies for `EditorMode` on `Document`.

```rust
// src/app.rs:29-38
pub struct App {
    document: Document,
    textarea: TextArea<'static>,
    context: GitContext,
    show_help: bool,
    rebase_lines: Vec<RebaseLine>,
    selected_rebase_idx: usize,
    selectable_indices: Vec<usize>,
    squash_log: Vec<String>,   // populated only in Squash context
}
```

`Document` gains one new field `mode: EditorMode` parallel to how `App` carries `squash_log`.

**Pattern: `Document::parse()` editable_index gate — lines 266–294**

The existing gate (lines 280–286) is the exact insertion point. Today only `ContentLine::Content` is pushed; in Plain mode `ContentLine::Comment` must also be pushed, and in Merge mode `ContentLine::ConflictMarker` must also be pushed.

```rust
// src/document.rs:280-287  — CURRENT gate to modify
for line in slice {
    let classified = classify_line(line, comment_char);
    let idx = lines.len();
    if let ContentLine::Content(_) = &classified {
        editable_index.push(idx);          // ← extend this match per EditorMode
    }
    lines.push(classified);
}
```

Replacement pattern (mode-aware match):

```rust
// NEW — replace the if-let above with:
let include = match &classified {
    ContentLine::Content(_) => true,
    ContentLine::Comment(_) => matches!(mode, EditorMode::Plain),
    ContentLine::ConflictMarker(_) => matches!(mode, EditorMode::Plain | EditorMode::Merge),
};
if include {
    editable_index.push(idx);
}
```

**Pattern: `editable_lines()` — lines 297–308**

Currently filters for `ContentLine::Content` only. In Plain mode all variants are already in `editable_index`, so `editable_lines()` should iterate `editable_index` to preserve the mode-aware set rather than re-filtering by variant:

```rust
// src/document.rs:297-308  — CURRENT
pub fn editable_lines(&self) -> Vec<String> {
    self.lines
        .iter()
        .filter_map(|l| {
            if let ContentLine::Content(s) = l {
                Some(s.clone())
            } else {
                None
            }
        })
        .collect()
}
```

After the change, drive from `editable_index` (already correct by construction):

```rust
// NEW
pub fn editable_lines(&self) -> Vec<String> {
    self.editable_index
        .iter()
        .map(|&i| match &self.lines[i] {
            ContentLine::Content(s)
            | ContentLine::Comment(s)
            | ContentLine::ConflictMarker(s) => s.clone(),
        })
        .collect()
}
```

**Pattern: `serialize()` — lines 335–353**

Current serialize re-injects non-Content lines verbatim by matching on `ContentLine::Comment | ContentLine::ConflictMarker`. In Plain mode all lines came from the textarea, so the loop must handle plain mode differently — emit `textarea_lines` for every line position, not just Content positions.

```rust
// src/document.rs:335-353  — CURRENT serialize
pub fn serialize(&self, textarea_lines: &[String]) -> String {
    let mut output = String::new();
    let mut content_idx: usize = 0;

    for line in &self.lines {
        match line {
            ContentLine::Content(_) => {
                output.push_str(&textarea_lines[content_idx]);
                content_idx += 1;
            }
            ContentLine::Comment(s) | ContentLine::ConflictMarker(s) => {
                output.push_str(s);   // verbatim re-inject
            }
        }
        output.push('\n');
    }
    output
}
```

Plain-mode serialize: D-02 says output is simply `textarea_lines.join("\n") + "\n"`. Add a mode branch at the top:

```rust
// NEW top-of-serialize branch
if matches!(self.mode, EditorMode::Plain) {
    let mut out = textarea_lines.join("\n");
    out.push('\n');
    return out;
}
// existing loop follows unchanged for Squash / Merge
```

---

### `src/app.rs` — `App::new()` mode propagation + subject-counter removal

**Pattern: mode-gated branch in `App::new()`**

Source: `src/app.rs` lines 47–76 — the existing `if matches!(context, GitContext::Squash)` block is the direct template for adding `EditorMode` selection.

```rust
// src/app.rs:47-76  — EXISTING squash branch (template)
let (squash_log, content_for_document) = if matches!(context, GitContext::Squash) {
    if let Some(header_end) = detect_squash_header(raw_content, comment_char) {
        // ... split header / message
        (header_lines, message_str)
    } else {
        (Vec::new(), raw_content.to_string())
    }
} else {
    (Vec::new(), raw_content.to_string())
};
```

New code selects `EditorMode` by matching `context` before calling `Document::parse`:

```rust
// NEW — inserted before Document::parse call (src/app.rs:78)
let editor_mode = match context {
    GitContext::Commit => EditorMode::Plain,
    GitContext::Merge  => EditorMode::Merge,
    GitContext::Squash => EditorMode::Squash,
    _                  => EditorMode::Plain,
};
let document = Document::parse(&content_for_document, comment_char, editor_mode);
```

**Pattern: `serialized_content()` — lines 211–226**

The squash branch (lines 214–221) demonstrates how mode-specific logic is expressed. The plain-mode case is the `else` arm (line 223–225) — `document.serialize(textarea.lines())` — which already works correctly once `Document::serialize` handles plain mode internally.

```rust
// src/app.rs:211-226  — CURRENT (no changes needed after Document handles it)
pub fn serialized_content(&self) -> String {
    if matches!(self.context, GitContext::Rebase) {
        serialize_rebase_todo(&self.rebase_lines)
    } else if matches!(self.context, GitContext::Squash) && !self.squash_log.is_empty() {
        let mut output = String::new();
        for line in &self.squash_log {
            output.push_str(line);
            output.push('\n');
        }
        output.push_str(&self.document.serialize(self.textarea.lines()));
        output
    } else {
        self.document.serialize(self.textarea.lines())
    }
}
```

No structural changes needed here — the Plain mode path is already the `else` branch. `Document::serialize` handles the distinction internally.

---

## Shared Patterns

### Enum Definition Style
**Source:** `src/context.rs` lines 4–11 and `src/document.rs` lines 1–9
**Apply to:** New `EditorMode` enum in `src/document.rs`

All enums use `#[derive(Debug, Clone, PartialEq)]`; small flag enums also add `Copy`. No `Default` derive is used in this codebase — default variant is chosen explicitly at construction.

### Mode-Aware Branching via `matches!`
**Source:** `src/app.rs` lines 47, 212, 214
**Apply to:** `Document::parse()`, `Document::serialize()`, `App::new()`

The project consistently uses `matches!(expr, Pattern)` for enum branch tests, not `== Variant` or `if let`. Follow this in all new branches.

```rust
// Pattern used everywhere
if matches!(context, GitContext::Squash) { ... }
if matches!(self.mode, EditorMode::Plain) { ... }
```

### Test Structure (inline `#[cfg(test)]` modules)
**Source:** `src/document.rs` lines 384–877 and `src/app.rs` lines 229–423
**Apply to:** New tests for `EditorMode`-gated behavior

All tests live in an inline `#[cfg(test)] mod tests { use super::*; ... }` block at the bottom of the file. New tests for `EditorMode::Plain` (commit mode), `EditorMode::Merge` (ConflictMarker editability), and serialize round-trips follow the existing test naming convention: `test_<function>_<scenario>`.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | All files being modified have direct self-analogs |

The subject-line counter removal (D-06) affects the renderer. Check `src/renderer.rs` for the counter render path — no pattern extraction needed; it is a deletion, not a new pattern.

---

## Metadata

**Analog search scope:** `src/` (document.rs, app.rs, context.rs, renderer.rs)
**Files read:** 4
**Key insight:** `EditorMode` on `Document` mirrors the existing `GitContext` enum pattern in `src/context.rs`. The `editable_index` gate in `Document::parse()` (lines 280–286) is the single insertion point that controls all downstream behavior — `editable_lines()`, `full_row_for_editable()`, and `serialize()` all follow from it automatically in non-Plain modes; Plain mode adds a short-circuit in `serialize()` per D-02.
