# Architecture Patterns

**Project:** gitmedit v1.1
**Domain:** TUI git editor — integrating new v1.1 features into existing 6-module codebase
**Researched:** 2026-04-07
**Confidence:** HIGH (all analysis derived from direct code inspection of src/*.rs)

---

## Existing Architecture (as-built, v1.0)

```
main.rs         CLI parsing, event loop, key dispatch, save/cancel exit
app.rs          App struct (state machine), Action enum, Outcome enum
context.rs      GitContext enum, detect_context() by filename
document.rs     Document (line classification / index / serialize),
                RebaseLine, RebaseAction, parse_rebase_todo, serialize_rebase_todo,
                detect_squash_header, read_comment_char
renderer.rs     Renderer (stateless), dispatches on GitContext, status bars, help overlay
terminal.rs     TerminalGuard (RAII raw-mode + alternate screen), panic hook
writer.rs       FileWriter::write_atomic()
```

### Current Renderer Layout

```
render()
  [Constraint::Min(0)]      ← content area
  [Constraint::Length(1)]   ← 1-line status bar

  Rebase     → render_rebase_table()       + render_rebase_status_bar()
  Squash     → render_squash_mode()        + render_squash_status_bar()
  _          → render_content()            + render_status_bar()

  (overlay)  → render_help_overlay()       (all modes, on top)
```

### Key Structural Facts for Integration

- `App::new()` signature: `(raw_content: &str, context: GitContext) -> Self`. No path stored in App.
- The path lives only in `main.rs` (`cli.path: PathBuf`). App is path-agnostic.
- `Cli` struct requires `path: PathBuf` (not optional) — `gitmedit` with no args panics via clap.
- `Document` classifies every line into `Content | Comment | ConflictMarker`. Only `Content` lines go into TextArea. `serialize()` interleaves them back.
- `terminal.rs` comment says "Does not use EnterAlternateScreen" but the code does `EnterAlternateScreen` — IO-06 is a documented regression where comment and code are inconsistent.
- `render_status_bar()` is shared by Commit / Merge / Unknown. Squash and Rebase each have their own dedicated status bar renderer (three total).

---

## Feature Integration Analysis

### Feature 1: Standalone Commit Mode

**What:** `gitmedit` invoked with no args in a git repo creates an empty COMMIT_EDITMSG, opens TUI, then runs `git commit -F <tmpfile>` on save.

**Integration points:**

| Touch point | Change | Detail |
|-------------|--------|--------|
| `main.rs` — `Cli` struct | MODIFY | `path: PathBuf` → `path: Option<PathBuf>` (clap `#[arg(required = false)]`) |
| `main.rs` — startup | MODIFY | When `path.is_none()`: call `git rev-parse --is-inside-work-tree`, create `NamedTempFile`, set `context = GitContext::Commit`, pass `""` as raw_content |
| `main.rs` — save handler | MODIFY | New `InvocationMode` enum branches save behavior |
| `main.rs` — cancel handler | MODIFY | Standalone cancel deletes tmpfile, exits 1 (no file write) |
| `app.rs` | NO CHANGE | App takes `(raw_content, context)` — standalone passes `("", GitContext::Commit)`, indistinguishable from git-invoked commit |
| `context.rs` | NO CHANGE | `GitContext::Commit` already covers this path |

**New type in `main.rs`:**

```rust
enum InvocationMode {
    GitManaged { path: PathBuf },
    Standalone { tmpfile: tempfile::NamedTempFile },
}
```

Save handler branches on this:
- `GitManaged`: current behavior — `FileWriter::write_atomic` + `process::exit(0)`
- `Standalone`: write content to tmpfile path, spawn `git commit -F <tmpfile_path>`, exit with git's exit code

**Git repo guard:** Before entering TUI in standalone mode, run `git rev-parse --is-inside-work-tree`. If non-zero exit, print error and exit 1 without opening TUI.

**Dependency:** The `tempfile` crate is already used in `writer.rs` tests. Add it as a proper runtime dependency in `Cargo.toml`.

---

### Feature 2: Plain Editor Default (Remove Comment Protection)

**What:** All lines become editable regardless of content. Remove the Content / Comment / ConflictMarker discrimination when not needed.

**Integration points:**

| Touch point | Change | Detail |
|-------------|--------|--------|
| `document.rs` — `Document` struct | MODIFY | Add `mode: EditorMode` field |
| `document.rs` — `classify_line()` | MODIFY | In `Plain` mode, always return `ContentLine::Content` |
| `document.rs` — `serialize()` | SIMPLIFY | In `Plain` mode, emit all lines directly from TextArea (no interleaving) |
| `app.rs` — `App::new()` | MODIFY | Determine `EditorMode` from context; pass to `Document::parse()` |
| `renderer.rs` — `render_content()` | LOW RISK | Comment/ConflictMarker branches remain; unreachable in plain mode — no change required |

**New enum in `document.rs`:**

```rust
pub enum EditorMode { Plain, GitAware }
```

When `EditorMode::Plain`, `classify_line()` always returns `ContentLine::Content`. The `editable_index` then equals `0..lines.len()`, and `serialize()` is a trivial line join.

**Decision rule in `app.rs`:**

```rust
let editor_mode = match context {
    GitContext::Unknown => EditorMode::Plain,
    _ => EditorMode::GitAware,
};
```

`GitContext::Unknown` is the case when gitmedit is called with an unrecognized filename (e.g., a custom script). All git-specific contexts (Commit, Merge, Rebase, Squash) retain git-aware behavior.

**Existing code safety:** All renderer pattern matches on `ContentLine` variants continue to compile. In plain mode they simply never match Comment or ConflictMarker branches — no behavioral regression.

---

### Feature 3: Nano-Style Chrome (Header Bar + Footer Command Bar)

**What:** Add a 1-line header at top showing mode label and filename; replace the current 1-line status bar with a consistent nano-style command bar across all modes.

**Integration points:**

| Touch point | Change | Detail |
|-------------|--------|--------|
| `renderer.rs` — `render()` layout | MODIFY | 2-constraint → 3-constraint layout |
| `renderer.rs` — NEW `render_header()` | NEW | Top bar: mode label (left) + filename (right) |
| `renderer.rs` — `render_footer()` | NEW / CONSOLIDATE | Replaces three current status bar renderers; dispatches on context |
| `app.rs` — `App` struct | MODIFY | Add `display_path: String` field |
| `main.rs` — `App::new()` call | MODIFY | Pass display path (filename component, or `"(new commit)"` in standalone mode) |

**New layout in `render()`:**

```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(1),   // header bar   ← NEW
        Constraint::Min(0),      // content area
        Constraint::Length(1),   // command bar  (was status bar)
    ])
    .split(frame.area());
// chunks[0] = header, chunks[1] = content, chunks[2] = footer
```

**Header bar (`render_header`):**
- Left: context label styled bold (e.g., "COMMIT", "MERGE", "REBASE", "SQUASH")
- Right: `app.display_path()` (e.g., `COMMIT_EDITMSG` or `(new commit)`)
- Background: same DarkGray used by existing status bars for visual continuity

**Footer consolidation:** The three current status bar functions have divergent content but identical structure. Replace with `render_footer(frame, app, area)` that dispatches internally:
- Commit/Merge/Unknown: char counter + blank-line warning + hotkeys
- Merge: adds conflict metadata when `merge_metadata` is present (see Feature 4)
- Squash: existing squash content
- Rebase: existing rebase content

This eliminates duplication and keeps nano-style consistency across all modes.

**Path access:** Rather than changing `Renderer::render()` signature, store `display_path: String` in `App`. The renderer already receives `&App` per frame — no new dependencies needed.

---

### Feature 4: Merge Commit Toolbar (Comment Parsing)

**What:** In `GitContext::Merge`, parse the comment block in `MERGE_MSG` to extract conflict count and affected file paths, then display this in the footer.

**Integration points:**

| Touch point | Change | Detail |
|-------------|--------|--------|
| `document.rs` — NEW `MergeMetadata` struct | NEW | `conflict_count`, `affected_files`, `branch_being_merged` |
| `document.rs` — NEW `parse_merge_metadata()` | NEW | Iterates `Document.lines()`, finds conflict comment patterns |
| `app.rs` — `App` struct | MODIFY | Add `merge_metadata: Option<MergeMetadata>` field |
| `app.rs` — `App::new()` | MODIFY | Call `parse_merge_metadata()` when `context == GitContext::Merge` |
| `renderer.rs` — `render_footer()` | MODIFY | Merge context branch reads `app.merge_metadata()` and renders file list |

**`MergeMetadata` struct (in `document.rs`):**

```rust
pub struct MergeMetadata {
    pub conflict_count: usize,
    pub affected_files: Vec<String>,
    pub branch_being_merged: Option<String>,
}
```

**Parsing target patterns in `MERGE_MSG`:**
```
Merge branch 'feature' into main        ← first content line, branch name extractable
# Conflicts:
#   path/to/file.rs
#   src/another.go
```

`parse_merge_metadata()` iterates `ContentLine::Comment` lines:
- `"# Conflicts:"` → sets a flag indicating subsequent indented comment lines are files
- `"#   path/..."` → appends to `affected_files`
- Falls out of the file-listing section when it hits a non-indented comment or content line

The function belongs in `document.rs` alongside `parse_rebase_todo` and `detect_squash_header` — the same document-parsing family. No new module needed.

**Parse once at construction:** `parse_merge_metadata()` is called once in `App::new()` and stored. The renderer reads `app.merge_metadata()` — no re-parsing per frame.

---

### Feature 5: Rebase Line Reordering

**What:** Shift+Up / Shift+Down physically moves the selected Action line up or down in the `rebase_lines` Vec.

**Integration points:**

| Touch point | Change | Detail |
|-------------|--------|--------|
| `app.rs` — `Action` enum | MODIFY | Add `MoveRebaseLineUp`, `MoveRebaseLineDown` variants |
| `app.rs` — `App::apply()` | MODIFY | Swap logic in `rebase_lines` Vec |
| `main.rs` — Rebase key dispatch | MODIFY | Map `(KeyCode::Up, KeyModifiers::SHIFT)` and `(KeyCode::Down, KeyModifiers::SHIFT)` to new Actions |
| `document.rs` | NO CHANGE | `Vec<RebaseLine>` is already mutable in App; no new parsing needed |
| `renderer.rs` | NO CHANGE | Rebase table reads `app.rebase_lines()` — reordering is invisible to renderer |

**Swap logic in `App::apply(MoveRebaseLineDown)`:**

```rust
// Get current and next selectable indices into rebase_lines
if let (Some(&cur_idx), Some(&next_idx)) = (
    self.selectable_indices.get(self.selected_rebase_idx),
    self.selectable_indices.get(self.selected_rebase_idx + 1),
) {
    self.rebase_lines.swap(cur_idx, next_idx);
    // selectable_indices values (positions in rebase_lines) are now swapped too
    self.selectable_indices.swap(
        self.selected_rebase_idx,
        self.selected_rebase_idx + 1,
    );
    self.selected_rebase_idx += 1;
}
```

**Critical detail:** When two adjacent Action lines swap positions in `rebase_lines`, their entries in `selectable_indices` also swap (because `selectable_indices` stores positions by index into `rebase_lines`). Both `rebase_lines.swap()` and `selectable_indices.swap()` must happen together.

**Non-action lines between action lines:** If Comment lines exist between two Action lines, `selectable_indices[i]` and `selectable_indices[i+1]` may not be adjacent in `rebase_lines`. The swap of the two Action entries in `rebase_lines` leaves Comment lines in place — which is correct behavior. The selection cursor tracks into `selectable_indices`, not `rebase_lines` directly, so the cursor position update (`selected_rebase_idx += 1`) is correct regardless of interstitial comments.

**Key binding:** Crossterm represents Shift+Up as `KeyEvent { code: KeyCode::Up, modifiers: KeyModifiers::SHIFT, .. }`. This is reliable on macOS/Linux terminals. Flag for cross-platform verification in Windows testing.

---

### Feature 6: Exec Line Argument Editing

**What:** Allow editing the command string of `RebaseLine::Action { action: RebaseAction::Exec, subject, .. }` lines.

**Integration points:**

| Touch point | Change | Detail |
|-------------|--------|--------|
| `app.rs` — `App` struct | MODIFY | Add `exec_edit_mode: bool`, `exec_edit_textarea: Option<TextArea<'static>>` |
| `app.rs` — `Action` enum | MODIFY | Add `EnterExecEdit`, `CommitExecEdit`, `CancelExecEdit` |
| `app.rs` — `App::apply()` | MODIFY | Handle exec edit lifecycle |
| `main.rs` — Rebase key dispatch | MODIFY | Enter key triggers `EnterExecEdit` when cursor is on an Exec line; in exec edit mode, route keys to `exec_edit_textarea`; Enter = commit, Esc = cancel |
| `renderer.rs` — NEW `render_exec_edit_overlay()` | NEW | Modal overlay with single-line TextArea, similar to `render_help_overlay()` |

**State machine:**

```
Normal rebase mode
  Enter on Exec line → EnterExecEdit
    exec_edit_textarea = TextArea::new(vec![current_subject])
    exec_edit_mode = true

ExecEditMode (in event loop)
  printable keys / backspace → routed to exec_edit_textarea
  Enter or Ctrl+S → CommitExecEdit
    rebase_lines[selected_idx].subject = exec_edit_textarea.lines()[0].clone()
    exec_edit_mode = false, exec_edit_textarea = None
  Esc → CancelExecEdit
    exec_edit_mode = false, exec_edit_textarea = None
```

**Rendering:** Use a floating modal overlay (same `centered_rect()` helper used by help overlay) rather than embedding a TextArea widget inside a Table cell. The Table cell approach would require row height changes during edit mode and complex widget positioning — the modal is simpler and consistent with the help overlay pattern already in the codebase.

**Single-line constraint:** Exec commands are inherently single-line. The overlay TextArea gets `single_line: true` behavior (or just use `lines()[0]` and ignore subsequent lines on commit).

**Identify exec line at selection:** `app.selected_rebase_line_idx()` returns the index into `rebase_lines`. Check if `rebase_lines[idx]` is `Action { action: Exec, .. }` before triggering `EnterExecEdit`.

---

## Component Boundaries After v1.1

```
main.rs         CLI (path: Option<PathBuf>), InvocationMode enum,
                startup git-repo detection, event loop, key dispatch,
                standalone commit flow

app.rs          App state machine
  fields:       document, textarea, context, show_help,
                rebase_lines, selected_rebase_idx, selectable_indices,
                squash_log,
                merge_metadata: Option<MergeMetadata>,     ← NEW
                exec_edit_mode: bool,                       ← NEW
                exec_edit_textarea: Option<TextArea>,       ← NEW
                display_path: String                        ← NEW
  actions:      Save, Cancel, Noop, Help, DismissHelp,
                CycleRebaseAction, MoveRebaseDown, MoveRebaseUp,
                MoveRebaseLineDown, MoveRebaseLineUp,        ← NEW
                EnterExecEdit, CommitExecEdit, CancelExecEdit ← NEW

context.rs      GitContext, detect_context()  [NO CHANGE]

document.rs     Document (+ EditorMode field),
                ContentLine, classify_line(),
                RebaseLine, RebaseAction, parse_rebase_todo, serialize_rebase_todo,
                detect_squash_header,
                EditorMode enum,                            ← NEW
                MergeMetadata struct,                       ← NEW
                parse_merge_metadata()                      ← NEW

renderer.rs     Renderer — 3-slot layout
  NEW:          render_header()
  CONSOLIDATED: render_footer() replaces 3 current status bars
  UNCHANGED:    render_content(), render_squash_mode(), render_rebase_table()
  NEW:          render_exec_edit_overlay()

terminal.rs     TerminalGuard  [FIX: reconcile IO-06 — comment vs code inconsistency]

writer.rs       FileWriter::write_atomic()  [NO CHANGE]
```

---

## Data Flow Changes

### Standalone commit mode

```
main() — path arg is None
  → git rev-parse --is-inside-work-tree  (fail = exit 1)
  → NamedTempFile::new()                 (tmpfile for commit message)
  → App::new("", GitContext::Commit, display_path="(new commit)")
  → [edit loop — identical to git-managed commit]
  → on Save:
      write content to tmpfile
      spawn: git commit -F <tmpfile.path()>
      exit with git's exit code
  → on Cancel:
      drop tmpfile (auto-deleted)
      exit 1
```

### Plain editor mode

```
main() detects GitContext::Unknown (unknown filename)
  → App::new(raw_content, ctx) determines EditorMode::Plain
  → Document::parse(raw, comment_char, EditorMode::Plain)
      all lines classified as ContentLine::Content
      editable_index = 0..lines.len()
  → TextArea gets all lines
  → on Save: serialize() emits all TextArea lines (no interleaving)
```

### Nano chrome layout

```
render(frame, app)
  chunks[0] (Length 1) → render_header(frame, app, chunks[0])
      Left:  context label ("COMMIT" / "MERGE" / etc.)
      Right: app.display_path()
  chunks[1] (Min 0)    → per-mode content dispatch (unchanged)
  chunks[2] (Length 1) → render_footer(frame, app, chunks[2])
      Commit/Unknown:  char counter + blank-line warning + hotkeys
      Merge:           conflict count + affected files + hotkeys
      Squash:          existing squash content
      Rebase:          existing rebase content
```

### Merge metadata flow

```
App::new() when context == GitContext::Merge
  → Document::parse() → doc
  → parse_merge_metadata(doc.lines()) → MergeMetadata
      - conflict_count: count of "# Conflicts:" indented entries
      - affected_files: Vec of paths from indented comment lines
      - branch_being_merged: extracted from first content line
  → stored in app.merge_metadata: Option<MergeMetadata>

render_footer() when context == Merge
  → reads app.merge_metadata()
  → renders e.g. "2 conflicts: file.rs, other.go  |  ^S Save  Esc Cancel"
```

### Rebase reordering flow

```
Shift+Down key event
  → Action::MoveRebaseLineDown → app.apply()
      rebase_lines.swap(selectable_indices[i], selectable_indices[i+1])
      selectable_indices.swap(i, i+1)
      selected_rebase_idx += 1
  → on next draw: render_rebase_table reads reordered rebase_lines
  → on Ctrl+S: serialize_rebase_todo(rebase_lines) → new order written to disk
```

---

## Suggested Build Order

Dependencies are strictly respected. Each step produces a compilable, testable increment.

| Step | Feature | Rationale |
|------|---------|-----------|
| 1 | Fix IO-06 (terminal.rs) | Zero risk; touches one file; unblocks honest cross-platform testing. Fix before any other work so baseline terminal behavior is known-good. |
| 2 | Plain editor default (document.rs + app.rs) | Simplifies Document model early. Subsequent features build on clean model. Low coupling — renderer unaffected. |
| 3 | Nano-style chrome (renderer.rs + app.rs) | Restructures renderer layout once. All subsequent footer/header additions land on the new 3-slot layout. Must precede merge toolbar (step 4 needs footer to exist). |
| 4 | Merge commit toolbar (document.rs + app.rs + renderer.rs) | Adds parsing family member to document.rs; renders in footer created in step 3. Depends on 3. |
| 5 | Rebase line reordering (app.rs + main.rs) | Isolated to Action enum + apply() + key dispatch. No renderer changes. Does not depend on 3 or 4 but benefits from clean codebase. |
| 6 | Exec line argument editing (app.rs + main.rs + renderer.rs) | Depends on rebase mode being stable (step 5). Adds overlay renderer similar to help overlay. |
| 7 | Standalone commit mode (main.rs + Cargo.toml) | Builds on plain editor (step 2). Safe to add late — it's a new code path in main.rs that doesn't affect existing modes. |
| 8 | Custom hotkey configuration | Cross-cutting addition easiest after all features are stable. Add `HotkeyConfig` struct, loaded at startup, replaces hardcoded key matches in event loop. |
| 9 | Cross-platform testing | Runs in parallel with 5–8 but gated on IO-06 fix (step 1). Windows terminal behavior for Shift+Up/Down requires empirical verification. |

**Critical dependency:** Step 3 (nano chrome) must precede step 4 (merge toolbar) because the toolbar renders in the footer that step 3 creates. All other steps are independent of each other.

---

## Anti-Patterns to Avoid

### Duplicating path state into App beyond display

The canonical file path for I/O lives in `main.rs`. Store only a `display_path: String` in `App` for rendering. Do not add `PathBuf` to App — it would couple the state machine to file I/O concerns.

### Merging exec-edit state into the main TextArea

The main `TextArea` owns the commit message. The exec-edit TextArea is transient and mode-specific. Keep it as `Option<TextArea>` in App, not a flag on the main textarea.

### Re-parsing MergeMetadata on every render frame

`parse_merge_metadata()` runs once in `App::new()`. Store the result in `app.merge_metadata`. The renderer reads it; it never calls parsing functions directly.

### Embedding a TextArea widget inside a Table cell for exec editing

The ratatui `Table` widget does not support interactive child widgets. Use a floating modal overlay (the `centered_rect()` helper already exists in renderer.rs) for exec edit — consistent with the help overlay pattern.

### Changing the `Renderer::render()` signature to accept path

The renderer signature `render(frame: &mut Frame, app: &App)` is clean. Store `display_path` in `App` rather than adding a third parameter — keeps renderer dependency on App only.

### Skipping `selectable_indices.swap()` when reordering

Swapping `rebase_lines` entries without also swapping the corresponding entries in `selectable_indices` will desync the selection cursor from the actual line positions. Both swaps must happen atomically in `apply()`.

---

## Module Modification Summary

| Module | Status | What Changes |
|--------|--------|-------------|
| `main.rs` | MODIFY | `Cli.path` → `Option<PathBuf>`, `InvocationMode` enum, standalone git commit flow, Shift+Up/Down bindings, Enter-on-exec binding, exec edit key routing |
| `app.rs` | MODIFY | `display_path: String` field, `merge_metadata: Option<MergeMetadata>` field, `exec_edit_mode` + `exec_edit_textarea` fields, new Action variants, swap logic for reorder, exec edit lifecycle in `apply()` |
| `context.rs` | NO CHANGE | GitContext variants cover all needed modes |
| `document.rs` | MODIFY | `EditorMode` enum, `Document::parse()` accepts mode param, `MergeMetadata` struct, `parse_merge_metadata()` function |
| `renderer.rs` | MODIFY | 3-slot layout in `render()`, `render_header()` (new), `render_footer()` (consolidates 3 status bars), `render_exec_edit_overlay()` (new) |
| `terminal.rs` | FIX | IO-06: make comment and code consistent on alternate screen behavior |
| `writer.rs` | NO CHANGE | Atomic write behavior is correct as-is |
| `Cargo.toml` | MODIFY | Add `tempfile` as runtime dep (currently only in dev-dependencies via writer tests) |

---

## Sources

- Direct code analysis: all 7 modules in `src/` read in full (main.rs, app.rs, context.rs, document.rs, renderer.rs, terminal.rs, writer.rs)
- Project context: `.planning/PROJECT.md`
- All findings are HIGH confidence — derived from actual v1.0 codebase, no inference required

---

*Architecture research for: gitmedit v1.1 feature integration*
*Researched: 2026-04-07*
