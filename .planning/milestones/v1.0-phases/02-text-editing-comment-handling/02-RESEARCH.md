# Phase 2: Text Editing + Comment Handling - Research

**Researched:** 2026-03-20
**Domain:** Rust TUI text editing with ratatui-textarea, comment preservation, conflict marker styling
**Confidence:** HIGH (verified against local ratatui-textarea 0.8.0 source, arboard docs, Phase 1 source)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Comment parsing**: Parse on load at startup; detect comment lines once, not during runtime
- **Storage model**: `Vec<ContentLine>` enum where each line is `Content(String)`, `Comment(String)`, or `ConflictMarker(String)`
- **Comment character**: Read `core.commentChar` from git config at startup (default `#`)
- **Comment rendering**: Comment lines displayed in different foreground color (e.g., DarkGray)
- **Comment preservation**: Comment lines written byte-for-byte on save; never modified
- **Text model**: `Vec<ContentLine>` enum, not single String or line metadata
- **Cursor tracking**: Line and column coordinates `(line: usize, col: usize)`
- **Undo/Redo**: Command history pattern — each keystroke stored as an Action, undo/redo rewind/replay
- **Mutation safety**: Cursor can move to comment lines (visual nav), but mutations blocked if target line is comment
- **ConflictMarker**: Separate `ContentLine::ConflictMarker` variant; detected line-by-line
- **Conflict editability**: Conflict marker lines (<<<<<<, ======, >>>>>>) are read-only; content lines between markers are fully editable
- **Conflict styling**: Conflict markers displayed with different background color (distinct from comments)
- **Basic edit ops**: Insert character, delete character (backspace/delete), arrow key navigation, Home/End, Ctrl+U (delete entire line)
- **Word ops**: Ctrl+W to delete previous word, Ctrl+D to delete next word
- **System clipboard**: Ctrl+C to copy, Ctrl+X to cut, Ctrl+V to paste from system clipboard (arboard or similar)
- **Text selection**: Shift+arrow keys (left/right/up/down) to select text
- **Line wrapping**: Hard wrap at terminal width during rendering only; no horizontal scroll
- **Target FPS**: 30+ FPS during typing (no perceived lag)
- **Large files**: No noticeable lag on files >10KB

### Claude's Discretion

- Exact styling colors for comments vs conflict markers (beyond background/foreground choice)
- Word boundary detection algorithm (what constitutes a "word" for Ctrl+W/D)
- Clipboard provider (arboard vs copypasta vs other crate)
- How to handle terminal resize mid-editing
- Exact panic/error handling for clipboard unavailability

### Deferred Ideas (OUT OF SCOPE)

- Line reordering in merge conflicts (move <<<< to choose sides) — Phase 2.1 enhancement if needed
- Regex-based search/replace — v2 feature per REQUIREMENTS.md (EDIT-10)
- Terminal resize handling — can defer graceful handling to Phase 3 if basic resizing works
- Customizable hotkeys — Phase 2 uses hardcoded shortcuts; config file support is v2 (CONFIG-01)
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CTX-03 | Editor reads `core.commentChar` config at startup (default: #) | §Comment Handling — `git config --get core.commentchar` subprocess approach |
| CTX-04 | Comment lines are parsed and stored separately from editable content | §Text Model Architecture — `ContentLine` enum design |
| CTX-05 | Comment lines are displayed but not editable | §Architecture — mutation guard on comment lines; custom renderer per-line styling |
| CTX-06 | Comment lines are preserved byte-for-byte on file write | §Serialization — reconstruct original interleaving order on save |
| EDIT-01 | User can insert/delete characters anywhere in the message | §ratatui-textarea API — `TextArea::input()` handles Insert/Backspace/Delete |
| EDIT-02 | User can move cursor with arrow keys | §ratatui-textarea API — arrow keys handled via `Input::Key::Up/Down/Left/Right` |
| EDIT-03 | User can delete line with Ctrl+U | §Key Conflict Resolution — Ctrl+U in ratatui-textarea is undo; must remap to `delete_line_by_head()` |
| EDIT-04 | User can move to start/end of line (Home/End) | §ratatui-textarea API — `Key::Home`/`Key::End` built-in via `CursorMove::Head`/`CursorMove::End` |
| EDIT-05 | User can create new lines (multiline messages supported) | §ratatui-textarea API — Enter/Ctrl+M inserts newline by default |
| EDIT-06 | Text wraps at terminal width (no horizontal scroll needed) | §Rendering — `Paragraph` with ratatui `Wrap` widget; ratatui-textarea wraps automatically |
| EDIT-07 | Undo/redo work for text edits | §Undo/Redo — ratatui-textarea `undo()`/`redo()` built-in; default Ctrl+Z/Ctrl+Y |
| COMMIT-04 | User can save with Ctrl+S | Phase 1 complete; Ctrl+S must be intercepted before passing to textarea |
| COMMIT-05 | User can cancel with Esc | Phase 1 complete; Esc must be intercepted before passing to textarea |
| MERGE-01 | Conflict markers (<<<<<<, ======, >>>>>>) detected and styled | §Conflict Detection — line prefix matching; custom renderer for ConflictMarker variant |
| MERGE-02 | Conflict markers are not editable (treated as comments) | §Mutation Guard — same protection as comment lines |
| MERGE-03 | User can edit the resolved message between markers | §Architecture — Content lines between markers remain editable in the text model |
| PERF-02 | Rendering updates happen at 30+ FPS when typing | §Performance — immediate-mode ratatui + no allocation per keypress = well above 30FPS |
| PERF-03 | No noticeable lag on large files (>10KB) | §Performance — `Vec<String>` per-line model; only visible lines rendered via scroll viewport |
</phase_requirements>

---

## Summary

Phase 2 adds full text editing capabilities on top of Phase 1's event loop skeleton. The central challenge is not the editing mechanics — `ratatui-textarea 0.8.0` handles those — but the comment/conflict-marker overlay: these lines must render with distinct styling yet their content must never be passed to the editing widget.

The critical finding is that **`ratatui-textarea` does not support per-line styling based on content type**. The widget's public API provides only global style, cursor-line style, and selection style. There is no hook to color comment lines gray or conflict-marker lines red from within the widget. This forces a **hybrid rendering architecture**: a custom `Renderer` draws comment/marker lines directly using ratatui `Paragraph` spans, while only editable `Content` lines are fed into `TextArea`.

Comment character reading is best done via spawning `git config --get core.commentchar`; this handles global, system, and local config hierarchy transparently with zero custom parsing logic and no new dependencies.

**Primary recommendation:** Use `ratatui-textarea` for all editing mechanics (insert, delete, undo/redo, word ops, selection, cursor). Build a parallel rendering path in `Renderer` that replaces the `TextArea` widget output for comment and conflict-marker lines with styled `Paragraph` spans. The `Document` struct maps between the full `Vec<ContentLine>` model and the `TextArea`'s `Vec<String>` of editable lines only.

---

## Standard Stack

### Core (all already in Cargo.toml)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ratatui-textarea | 0.8.0 | Multi-line text editing, undo/redo, selection, word ops | Already in use; provides all EDIT-01..EDIT-07 mechanics |
| ratatui | 0.30.0 | TUI rendering, Paragraph, Layout, Span, Line, Style | Already in use |
| crossterm | 0.29.0 | Key events, terminal resize events, raw mode | Already in use |

### New Dependency

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| arboard | 3.6.1 | System clipboard get/set for Ctrl+C/X/V | Clipboard ops only; lazily initialized; errors are non-fatal |

### No New Dependencies Needed For

| Problem | Solution Without New Crate |
|---------|---------------------------|
| Comment char detection | `std::process::Command::new("git").args(["config","--get","core.commentchar"])` |
| Word boundary (Ctrl+W/D) | ratatui-textarea `delete_word()` / `delete_next_word()` built-in |
| Conflict marker detection | `line.starts_with("<<<<<<")` / `"======"` / `">>>>>>"` — plain `str::starts_with` |
| Undo/redo | ratatui-textarea `undo()` / `redo()` built-in (history: 50 entries default) |
| Text selection | ratatui-textarea `start_selection()`, shift-aware `move_cursor_with_shift()` |

**Installation — add to Cargo.toml:**
```toml
arboard = "3.6.1"
```

**Version verification (performed 2026-03-20):**
```
arboard 3.6.1  — confirmed current via cargo search
ratatui-textarea 0.8.0 — confirmed in local registry (published 2026-02-21)
```

---

## Architecture Patterns

### Recommended Project Structure

The Phase 1 structure (`app.rs`, `renderer.rs`, `main.rs`) is extended:

```
src/
├── main.rs          # event loop — intercept Ctrl+S/Esc/clipboard before textarea
├── app.rs           # App struct owns Document + TextArea; routes actions
├── document.rs      # NEW: Vec<ContentLine> model, cursor mapping, serialization
├── parser.rs        # NEW: raw file bytes → Document (comment char detection)
├── renderer.rs      # EXTENDED: custom per-line styled output (comment/marker/content)
├── input.rs         # NEW: KeyEvent → Action enum mapping
├── context.rs       # UNCHANGED from Phase 1
├── terminal.rs      # UNCHANGED from Phase 1
└── writer.rs        # EXTENDED: serialize Document (editable + comments in order)
```

### Pattern 1: ContentLine Enum (the core data model)

**What:** Every line in the file is typed at parse time. The type is immutable for the session.

```rust
// Source: Phase 2 design decision; confirmed against CONTEXT.md
pub enum ContentLine {
    Content(String),        // Editable — fed into TextArea
    Comment(String),        // Read-only — rendered gray, written verbatim
    ConflictMarker(String), // Read-only — rendered with distinct background, written verbatim
}
```

**When to use:** On every line during `parse()`. Type is set once, never changes.

**Serialization:** Iterate `Vec<ContentLine>` in order, write each variant's inner `String` followed by `\n`. This guarantees byte-for-byte comment preservation (CTX-06).

### Pattern 2: TextArea Feeds Only Editable Lines

**What:** The `Document` maintains a mapping between the full `Vec<ContentLine>` index and the editable-only index that `TextArea` uses internally.

```rust
// Source: inferred from ratatui-textarea 0.8.0 src/textarea.rs
pub struct Document {
    lines: Vec<ContentLine>,
    // Maps TextArea row → full lines index, for cursor re-mapping
    editable_index: Vec<usize>,
}

impl Document {
    pub fn editable_lines(&self) -> Vec<String> {
        self.lines.iter().filter_map(|l| match l {
            ContentLine::Content(s) => Some(s.clone()),
            _ => None,
        }).collect()
    }

    pub fn full_row_for_editable(&self, editable_row: usize) -> usize {
        self.editable_index[editable_row]
    }
}
```

**Implication:** `TextArea` is initialized with `editable_lines()`. On save, the planner must reconstruct the full `Vec<ContentLine>` by merging TextArea's final lines back into the non-editable lines at their original positions.

### Pattern 3: Custom Renderer for Per-Line Styling

**What:** Since `ratatui-textarea` cannot style comment/conflict lines differently, the `Renderer` bypasses the textarea widget for those lines and renders them directly as styled `Paragraph` spans.

**Critical API facts from ratatui-textarea 0.8.0 source:**
- `textarea.lines()` returns `&[String]` — the editable content
- `textarea.cursor()` returns `(usize, usize)` — row/col in editable coordinates
- The widget's internal `text_widget()` only renders lines it knows about

**Recommended approach:** Build a **unified scroll viewport** over the full `Vec<ContentLine>`. For each visible row, match on the variant:
- `Content`: render the corresponding TextArea line (with cursor highlight if this is the active row)
- `Comment`: render as `Span::styled(text, Style::default().fg(Color::DarkGray))`
- `ConflictMarker`: render as `Span::styled(text, Style::default().bg(Color::DarkRed).fg(Color::White))`

This approach replaces using `frame.render_widget(&textarea, area)` directly. Instead, the renderer manually draws each line.

**Alternative (simpler but limited):** Use `textarea` widget as-is and overlay a separate `Paragraph` widget for comment/marker lines. This risks z-order and cursor positioning complications. Not recommended.

### Pattern 4: Key Interception Before Textarea

**What:** Several keys must be intercepted in `main.rs` (or `input.rs`) before the event reaches `TextArea::input()`:

| Key | Default textarea behavior | Required behavior |
|-----|--------------------------|-------------------|
| `Ctrl+S` | No default mapping | Save and exit — must intercept |
| `Esc` | No default mapping | Cancel and exit — must intercept |
| `Ctrl+U` | **Undo** (in ratatui-textarea) | Delete entire line (EDIT-03) — must intercept and call `textarea.delete_line_by_head()` then `textarea.delete_line_by_end()` |
| `Ctrl+C` | **Copy selection** (internal yank) | System clipboard copy — must intercept |
| `Ctrl+X` | **Cut selection** (internal yank) | System clipboard cut — must intercept |
| `Ctrl+V` | PageDown scroll | System clipboard paste — must intercept |

**CRITICAL CONFLICT — Ctrl+U:** ratatui-textarea maps `Ctrl+U` to `undo()`, not line deletion. The project requires `Ctrl+U = delete entire line` (nano convention). Resolution: use `textarea.input_without_shortcuts()` as the fallback for unhandled keys, OR intercept `Ctrl+U` and call `delete_line_by_head()` + `delete_line_by_end()` manually.

**CRITICAL CONFLICT — Undo/Redo:** ratatui-textarea uses `Ctrl+U` for undo and `Ctrl+R` for redo. Phase 2 remaps `Ctrl+U` to line-delete. Standard undo should be `Ctrl+Z` (common) — must map to `textarea.undo()`. Standard redo should be `Ctrl+Y` — must map to `textarea.redo()`.

**Pattern:**
```rust
// Source: ratatui-textarea 0.8.0 src/textarea.rs lines 270-680
match event {
    // App-level intercepts first
    Event::Key(k) if k.code == KeyCode::Char('s') && k.modifiers == KeyModifiers::CONTROL => {
        // → Action::Save
    }
    Event::Key(k) if k.code == KeyCode::Esc => {
        // → Action::Cancel
    }
    Event::Key(k) if k.code == KeyCode::Char('u') && k.modifiers == KeyModifiers::CONTROL => {
        // → delete entire line (NOT undo)
        textarea.move_cursor(CursorMove::Head);
        textarea.delete_line_by_end();
    }
    Event::Key(k) if k.code == KeyCode::Char('z') && k.modifiers == KeyModifiers::CONTROL => {
        textarea.undo();
    }
    Event::Key(k) if k.code == KeyCode::Char('y') && k.modifiers == KeyModifiers::CONTROL => {
        textarea.redo();
    }
    Event::Key(k) if k.code == KeyCode::Char('c') && k.modifiers == KeyModifiers::CONTROL => {
        // system clipboard copy
        if let Some(selected) = get_selection_text(&textarea) {
            if let Ok(mut clip) = arboard::Clipboard::new() {
                let _ = clip.set_text(selected);
            }
        }
    }
    Event::Key(k) if k.code == KeyCode::Char('v') && k.modifiers == KeyModifiers::CONTROL => {
        // system clipboard paste
        if let Ok(mut clip) = arboard::Clipboard::new() {
            if let Ok(text) = clip.get_text() {
                textarea.insert_str(text);
            }
        }
    }
    _ => {
        // Let textarea handle remaining keys
        textarea.input(event);
    }
}
```

### Pattern 5: Mutation Guard for Read-Only Lines

**What:** Before applying any edit action, check whether the current cursor is on a `Comment` or `ConflictMarker` line. The cursor in `TextArea` operates only on editable lines, so this check is at the `Document` level using the editable-to-full mapping.

The simplest guard: editable lines map 1:1 to `Content` variants only, since `Comment` and `ConflictMarker` are never passed to `TextArea`. The cursor can never land on a comment row within `TextArea` — because comment lines are not in the textarea. Visual cursor display over comments is a rendering concern, not a TextArea state.

**Rendering cursor on non-editable lines:** When the user presses Up/Down and the visual cursor would pass through a comment line, the `Document` must decide: either skip over comment lines (jump past them), or allow the cursor to park on a comment line visually while textarea remains on the nearest editable line. The CONTEXT.md decision is "cursor can move to any line including comments for visual feedback." This requires maintaining a separate `visual_cursor: usize` in `Document` that tracks the full-line index, while `TextArea`'s cursor reflects only the editable index.

### Anti-Patterns to Avoid

- **Feeding all lines to TextArea:** Never include `Comment` or `ConflictMarker` lines in TextArea's data. The widget would allow editing them.
- **Synchronizing textarea.lines() back on every keystroke:** Expensive if done naively for large files. Do it only on save. Keep `Document` as the truth and `TextArea` as the edit buffer.
- **Using `textarea.input()` for Ctrl+U without intercepting:** ratatui-textarea maps Ctrl+U to undo, not line delete. Not intercepting this will break nano-style muscle memory.
- **Calling `arboard::Clipboard::new()` on every keypress:** Clipboard construction involves system calls. Cache or construct per-clipboard-operation only.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Character insert / delete | Manual `String::insert`/`String::remove` with cursor tracking | `textarea.insert_char()`, `textarea.delete_char()`, `textarea.delete_next_char()` | ratatui-textarea handles Unicode char width, selection, undo history automatically |
| Word delete | Manual whitespace scan | `textarea.delete_word()`, `textarea.delete_next_word()` | Built-in; handles Unicode word boundaries |
| Undo/Redo stack | `VecDeque<Snapshot>` with full line clones | `textarea.undo()`, `textarea.redo()` | ratatui-textarea uses a command-history approach with per-edit `EditKind` — no full clone per keypress |
| Selection tracking | Manual `selection_start: Option<(usize, usize)>` | `textarea.start_selection()`, `textarea.cancel_selection()`, `textarea.is_selecting()` | Built-in; integrates with copy/cut |
| Internal clipboard (yank) | Separate `yank: String` | `textarea.copy()`, `textarea.paste()` | Built-in yank buffer; `yank_text()` returns the yanked string for bridging to system clipboard |
| Scroll viewport | Manual top-row calculation | TextArea handles `Viewport` automatically | Built-in scroll tracking with cursor-following |
| Comment char parsing | Parse INI-like `.git/config` manually | `Command::new("git").args(["config","--get","core.commentchar"])` | git binary handles full config hierarchy (system/global/local); zero parsing complexity |

**Key insight:** ratatui-textarea eliminates 80% of the text buffer complexity. The unique value-add of Phase 2 is the Comment/ConflictMarker overlay — that is what custom code should focus on.

---

## Common Pitfalls

### Pitfall 1: Ctrl+U Undo vs Line Delete Conflict

**What goes wrong:** If the event is passed directly to `textarea.input()`, `Ctrl+U` triggers `textarea.undo()` — the opposite of the intended "delete line" behavior expected by nano users.

**Why it happens:** ratatui-textarea maps `Ctrl+U` to undo following Emacs convention. gitmedit follows nano convention.

**How to avoid:** Intercept `Ctrl+U` in the event loop before calling `textarea.input()`. Call `textarea.move_cursor(CursorMove::Head)` then `textarea.delete_line_by_end()` to delete the entire line content.

**Warning signs:** Tests show Ctrl+U adds entries to undo history instead of deleting the current line.

### Pitfall 2: System Clipboard Failure in Headless/SSH Environments

**What goes wrong:** `arboard::Clipboard::new()` returns `Err(...)` when there is no display server (SSH sessions, Docker containers, CI). If this error is propagated as fatal, the editor crashes.

**Why it happens:** arboard on Linux requires X11 or Wayland to be available. On macOS it works without a display. In headless environments there is no clipboard target.

**How to avoid:** Wrap all clipboard operations in `if let Ok(mut clip) = arboard::Clipboard::new()`. Clipboard failure is a silent no-op, not a fatal error. Log the error in debug mode only. CONTEXT.md marks the exact error behavior as Claude's discretion.

**Warning signs:** Editor panics or exits with an error when run over SSH.

### Pitfall 3: Comment Lines Leaking Into TextArea

**What goes wrong:** A `Comment(s)` or `ConflictMarker(s)` variant's string is accidentally included in the `Vec<String>` passed to `TextArea::new()`. The user can then edit the comment text.

**Why it happens:** Forgetting to filter during `Document::editable_lines()` construction, or re-building the TextArea on every render from the full line list.

**How to avoid:** `Document::editable_lines()` must filter with `filter_map` to include only `ContentLine::Content` variants. Add a unit test: parse a file with comments, verify `editable_lines().len() < total lines`.

**Warning signs:** CTX-05 test fails (user can type into a comment line).

### Pitfall 4: Save Serialization Loses Comment Lines

**What goes wrong:** On save, `writer.rs` calls `textarea.lines()` and writes those directly to disk. All comment and conflict-marker lines are lost because they were never in the textarea.

**Why it happens:** The natural approach of "get lines from textarea and write" works in Phase 1 where all content is editable, but breaks in Phase 2.

**How to avoid:** Save must go through `Document::serialize()`. The function walks `Vec<ContentLine>` in order; for `Content` variants it takes the corresponding line from `textarea.lines()` (mapping by index); for `Comment`/`ConflictMarker` it writes the original stored string.

**Warning signs:** CTX-06 test fails (comments missing from saved file).

### Pitfall 5: Core.commentChar = "auto" Not Handled

**What goes wrong:** `git config --get core.commentchar` returns the literal string `"auto"` when the user has set `core.commentChar = auto`. Treating `"auto"` as the comment character causes every line to be non-comment.

**Why it happens:** git's `auto` mode selects a character dynamically based on message content — it starts with `#` and changes if `#` already appears in the message.

**How to avoid:** If the subprocess output is exactly `"auto"`, fall back to `#` for Phase 2 (documented limitation per REQUIREMENTS.md — `core.commentChar = auto` detection is explicitly out of scope for v1). Log a warning in debug mode.

**Warning signs:** Files where the user has set `core.commentChar = auto` have no comments detected.

### Pitfall 6: TextArea Viewport and Custom Renderer Desync

**What goes wrong:** If a custom per-line renderer is built without respecting TextArea's internal scroll offset, the visual position of the cursor and the highlighted cursor-line can drift out of sync.

**Why it happens:** TextArea maintains scroll state in an atomic `Viewport`. If you render lines independently, you lose this tracking.

**How to avoid:** Either use `TextArea` widget as-is (sacrificing per-line styling), or read `textarea.viewport` scroll_top and match it in the custom renderer. The cleanest solution for Phase 2 is rendering the TextArea widget for its editable area and overlaying styled comment/marker lines in adjacent Rects via Layout. See §Architecture Patterns for the layout strategy.

---

## Code Examples

Verified patterns from ratatui-textarea 0.8.0 source.

### Creating TextArea from Content Lines Only

```rust
// Source: ratatui-textarea 0.8.0 src/textarea.rs, From<I> impl
use ratatui_textarea::TextArea;

fn make_textarea(document: &Document) -> TextArea<'static> {
    let editable: Vec<String> = document.editable_lines();
    let mut ta = TextArea::new(editable);
    ta.set_cursor_line_style(Style::default()); // remove default underline
    ta
}
```

### Handling Key Events With Interception

```rust
// Source: ratatui-textarea 0.8.0 src/textarea.rs::input(), lines 270-679
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui_textarea::{Input, Key};

fn handle_event(event: Event, textarea: &mut TextArea, app: &mut App) -> Action {
    match &event {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            match (key.code, key.modifiers) {
                (KeyCode::Char('s'), KeyModifiers::CONTROL) => return Action::Save,
                (KeyCode::Esc, _)                           => return Action::Cancel,
                (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                    // nano: delete entire line — NOT undo
                    textarea.move_cursor(CursorMove::Head);
                    textarea.delete_line_by_end();
                    return Action::Noop;
                }
                (KeyCode::Char('z'), KeyModifiers::CONTROL) => {
                    textarea.undo();
                    return Action::Noop;
                }
                (KeyCode::Char('y'), KeyModifiers::CONTROL) => {
                    textarea.redo();
                    return Action::Noop;
                }
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    // Bridge internal yank to system clipboard
                    textarea.copy();
                    if let Ok(mut clip) = arboard::Clipboard::new() {
                        let _ = clip.set_text(textarea.yank_text().to_string());
                    }
                    return Action::Noop;
                }
                (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
                    if let Ok(mut clip) = arboard::Clipboard::new() {
                        if let Ok(text) = clip.get_text() {
                            textarea.insert_str(text);
                        }
                    }
                    return Action::Noop;
                }
                _ => {}
            }
        }
        Event::Resize(_, _) => return Action::Noop, // terminal.draw() handles autoresize
        _ => {}
    }
    // Pass remaining events to textarea
    textarea.input(event);
    Action::Noop
}
```

### Comment Line Detection

```rust
// Source: design decision from CONTEXT.md; pattern from REQUIREMENTS.md
fn classify_line(line: &str, comment_char: char) -> ContentLine {
    let trimmed = line; // Do NOT trim — preserve indentation
    if trimmed.starts_with(comment_char) {
        ContentLine::Comment(line.to_string())
    } else if trimmed.starts_with("<<<<<<")
           || trimmed.starts_with("======")
           || trimmed.starts_with(">>>>>>")
    {
        ContentLine::ConflictMarker(line.to_string())
    } else {
        ContentLine::Content(line.to_string())
    }
}
```

### Reading core.commentChar

```rust
// Source: `git config` docs; subprocess approach confirmed appropriate for single startup call
fn read_comment_char() -> char {
    let output = std::process::Command::new("git")
        .args(["config", "--get", "core.commentchar"])
        .output();
    match output {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout);
            let trimmed = s.trim();
            if trimmed == "auto" || trimmed.is_empty() {
                '#' // v1 limitation: treat auto as default
            } else {
                trimmed.chars().next().unwrap_or('#')
            }
        }
        _ => '#', // git not in PATH or no config set — use default
    }
}
```

### Document Serialization (preserves comments)

```rust
// Source: design decision from CONTEXT.md CTX-06 requirement
fn serialize(lines: &[ContentLine], textarea_lines: &[String]) -> String {
    let mut content_idx = 0;
    let mut out = String::new();
    for line in lines {
        match line {
            ContentLine::Content(_) => {
                out.push_str(&textarea_lines[content_idx]);
                content_idx += 1;
            }
            ContentLine::Comment(s) | ContentLine::ConflictMarker(s) => {
                out.push_str(s);
            }
        }
        out.push('\n');
    }
    out
}
```

### Per-Line Styled Rendering (hybrid approach)

```rust
// Source: ratatui 0.30 Span/Line/Text API
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

fn render_line(line: &ContentLine, is_cursor_line: bool) -> Line<'_> {
    match line {
        ContentLine::Content(s) => {
            let style = if is_cursor_line {
                Style::default().add_modifier(ratatui::style::Modifier::UNDERLINED)
            } else {
                Style::default()
            };
            Line::styled(s.as_str(), style)
        }
        ContentLine::Comment(s) => {
            Line::from(Span::styled(s.as_str(), Style::default().fg(Color::DarkGray)))
        }
        ContentLine::ConflictMarker(s) => {
            Line::from(Span::styled(
                s.as_str(),
                Style::default().bg(Color::Red).fg(Color::White),
            ))
        }
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| tui-rs tui-textarea | ratatui-textarea 0.8.0 (official ratatui fork) | 2026-02-21 | Use ratatui-textarea, not tui-textarea |
| `textarea.widget()` call | `frame.render_widget(&textarea, area)` | v0.5.3 | `.widget()` is deprecated — pass `&textarea` directly |
| `TextArea::from(text.lines())` | Same — still the canonical init pattern | — | Stable API |
| undo = `Ctrl+Z`, redo = `Ctrl+Y` | ratatui-textarea defaults: `Ctrl+U` undo, `Ctrl+R` redo | Library default | Must remap for Phase 2 |

**Deprecated/outdated:**
- `textarea.widget()`: deprecated since ratatui-textarea 0.5.3. Use `&textarea` directly in `frame.render_widget()`.
- `tui-textarea` crate (rhysd's original): last published Oct 2024. Use `ratatui-textarea` (ratatui org fork) instead.

---

## Open Questions

1. **Visual cursor on non-editable lines**
   - What we know: CONTEXT.md says cursor can visually park on comment lines for "visual feedback"
   - What's unclear: Whether this requires a separate `visual_cursor` in `Document` or if the cursor simply skips over comment lines (simpler)
   - Recommendation: Plan Wave 1 with skip-over behavior (simpler, less state). Upgrade to visual parking in a later wave only if user feedback demands it.

2. **Conflict-marker line suffix format**
   - What we know: `<<<<<<<`, `=======`, `>>>>>>>` are 7 chars each (standard git) but some tools use 6 chars
   - What's unclear: Should detection be `starts_with("<<<<<<")` (6 chars, catches both) or exact match?
   - Recommendation: Use `starts_with("<<<<<<")`, `starts_with("======")`, `starts_with(">>>>>>")` to match both 6-char and 7-char variants. This is a 2-line implementation decision; plan it as a named constant.

3. **arboard on macOS with no clipboard access (sandbox)**
   - What we know: arboard works on macOS without a display server. The error case is mainly headless Linux.
   - What's unclear: If gitmedit is run in a sandboxed terminal (e.g., some CI environments), clipboard may silently fail.
   - Recommendation: All clipboard ops are wrapped in `if let Ok(...)` — this is sufficient for v1. Document the degraded-clipboard behavior in user-facing error output as "clipboard unavailable, copy to system clipboard disabled."

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | none — inline `#[cfg(test)]` modules |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CTX-03 | comment_char defaults to `#` when git config absent | unit | `cargo test --lib document::tests::comment_char_default` | ❌ Wave 0 |
| CTX-04 | Comment lines parsed into Comment variant | unit | `cargo test --lib document::tests::parse_comment_lines` | ❌ Wave 0 |
| CTX-05 | Comment lines not present in `editable_lines()` | unit | `cargo test --lib document::tests::editable_lines_excludes_comments` | ❌ Wave 0 |
| CTX-06 | Roundtrip: parse then serialize preserves bytes | unit | `cargo test --lib writer::tests::roundtrip_preserves_comments` | ❌ Wave 0 |
| EDIT-01 | Insert char advances cursor and modifies content | unit | `cargo test --lib app::tests::insert_char_modifies_content` | ❌ Wave 0 |
| EDIT-02 | Arrow keys move cursor | unit | `cargo test --lib app::tests::arrow_keys_move_cursor` | ❌ Wave 0 |
| EDIT-03 | Ctrl+U deletes current line | unit | `cargo test --lib app::tests::ctrl_u_deletes_line` | ❌ Wave 0 |
| EDIT-04 | Home/End moves to line start/end | unit | `cargo test --lib app::tests::home_end_keys` | ❌ Wave 0 |
| EDIT-05 | Enter inserts newline | unit | `cargo test --lib app::tests::enter_inserts_newline` | ❌ Wave 0 |
| EDIT-07 | Ctrl+Z undoes last edit | unit | `cargo test --lib app::tests::ctrl_z_undoes` | ❌ Wave 0 |
| MERGE-01 | Conflict markers detected and stored as ConflictMarker | unit | `cargo test --lib document::tests::conflict_markers_detected` | ❌ Wave 0 |
| MERGE-02 | ConflictMarker lines not present in editable_lines() | unit | `cargo test --lib document::tests::conflict_markers_not_editable` | ❌ Wave 0 |
| CTX-06 | Conflict markers preserved verbatim in serialization | unit | `cargo test --lib writer::tests::roundtrip_preserves_markers` | ❌ Wave 0 |
| PERF-02 | Rendering benchmark does not block >33ms per frame | manual | Start editor, type rapidly, confirm no visual lag | manual only |
| PERF-03 | Load 10KB file and render | manual | `printf '#%s\n' {1..500} > /tmp/large_commit && gitmedit /tmp/large_commit` | manual only |

### Sampling Rate

- **Per task commit:** `cargo test --lib`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `src/document.rs` — `ContentLine` enum, `Document` struct, `parse()`, `editable_lines()`, `editable_index`, `serialize()`, test module — covers CTX-03, CTX-04, CTX-05, CTX-06, MERGE-01, MERGE-02
- [ ] `src/parser.rs` — `read_comment_char()`, `Parser::parse()` — covers CTX-03, CTX-04
- [ ] `src/input.rs` — `Action` enum extension, `handle_event()` — covers EDIT-01..EDIT-07
- [ ] `src/app.rs` — refactor to hold `Document` + `TextArea` — covers all EDIT-* and MERGE-*
- [ ] `src/writer.rs` — extend `write_atomic` to call `Document::serialize()` — covers CTX-06

---

## Sources

### Primary (HIGH confidence)

- `ratatui-textarea 0.8.0` local source `/Users/jilizart/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ratatui-textarea-0.8.0/src/textarea.rs` — confirmed full public API, key mappings (Ctrl+U=undo, Ctrl+R=redo, Ctrl+W=delete_word, Shift+arrows=selection), `input()` method, `input_without_shortcuts()`, `undo()`/`redo()`, `copy()`/`cut()`/`paste()`, `yank_text()`, `lines()`, `cursor()`
- `ratatui-textarea 0.8.0` local source `src/history.rs` — confirmed `EditKind` enum; undo uses per-edit inverse, not full snapshot clone per keypress
- `ratatui-textarea 0.8.0` local source `src/widget.rs` — confirmed `draw()` calls `autoresize()` internally; viewport handling
- `ratatui-core-0.1.0` local source `src/terminal/terminal.rs` line 312 — confirmed `autoresize()` is called inside `draw()`; `Event::Resize` need not trigger manual buffer resize
- `ratatui-textarea 0.8.0` `examples/minimal.rs` — confirmed `frame.render_widget(&textarea, area)` is the current pattern (not `.widget()`)
- Phase 1 source `src/app.rs`, `src/renderer.rs`, `src/main.rs`, `src/writer.rs` — confirmed existing integration points, `App::apply()` returns `Outcome` enum, `write_atomic` pattern

### Secondary (MEDIUM confidence)

- [arboard docs.rs Clipboard struct](https://docs.rs/arboard/latest/arboard/struct.Clipboard.html) — confirmed `new()`, `get_text()`, `set_text()` API; platform support including macOS/Linux
- [git/git commit 84c9dc2](https://github.com/git/git/commit/84c9dc2c5a2d34351a06554af32501d4f99990e9) — confirmed `core.commentChar=auto` algorithm: starts with `#`, selects alternative if message contains `#` lines
- WebSearch crossterm resize — confirmed `terminal.draw()` calls `autoresize()` automatically; `Event::Resize` just needs a re-render call
- `cargo search arboard` — confirmed current version 3.6.1

### Tertiary (LOW confidence)

- WebSearch arboard headless/SSH behavior — known failure mode; `Clipboard::new()` returns `Err` without display server; specific error variants not verified against current arboard 3.6 source

---

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — ratatui-textarea source read directly from local registry; arboard API confirmed from docs.rs
- Architecture: HIGH — ContentLine enum, TextArea feeding pattern, key interception all derived from direct source inspection
- Key conflict (Ctrl+U): HIGH — confirmed by reading ratatui-textarea 0.8.0 `src/textarea.rs` lines 574-580 where `Ctrl+U` maps to `self.undo()`
- Pitfalls: HIGH (code paths), MEDIUM (arboard headless) — primary paths from source; clipboard failure mode from WebSearch
- Conflict marker format: MEDIUM — `<<<<<<<` 7-char standard is well-known; 6-char variant from general knowledge

**Research date:** 2026-03-20
**Valid until:** 2026-06-20 (ratatui-textarea 0.8 is current as of 2026-02-21; API expected stable for several months)
