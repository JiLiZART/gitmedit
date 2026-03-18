# Architecture Research

**Domain:** TUI git editor (terminal process, not git client)
**Researched:** 2026-03-18
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Entry Point (main.rs)                    │
│  argv[1] = file path from git  ·  exit code → git           │
├─────────────────────────────────────────────────────────────┤
│                     Context Layer                            │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────────┐  │
│  │ FileDetector │  │ FileParser   │  │  GitContext enum  │  │
│  │              │  │              │  │  Commit/Merge/    │  │
│  │ path → type  │  │ raw → model  │  │  Rebase/Squash    │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬──────────┘  │
│         └─────────────────┴──────────────────-┘              │
├─────────────────────────────────────────────────────────────┤
│                     Application Layer                        │
│  ┌─────────────────────────────────────────────────────┐     │
│  │                    App (state owner)                 │     │
│  │  mode: EditMode | ConfirmCancel                      │     │
│  │  doc: Document (lines + comments + cursor)           │     │
│  │  context: GitContext                                 │     │
│  └─────────────────────────────────────────────────────┘     │
├─────────────────────────────────────────────────────────────┤
│                     Event Loop Layer                         │
│  ┌──────────────────┐        ┌──────────────────────────┐    │
│  │   InputHandler   │        │      Renderer            │    │
│  │                  │        │                          │    │
│  │  crossterm events│──────→ │  ratatui terminal.draw() │    │
│  │  → Action enum   │        │  frame per tick          │    │
│  └──────────────────┘        └──────────────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                     Output Layer                             │
│  ┌──────────────────────────────────────────────────────┐    │
│  │  FileWriter: serialize Document → original file path │    │
│  │  ExitCode: 0 (saved) | 1 (cancelled/empty warning)   │    │
│  └──────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Communicates With |
|-----------|----------------|-------------------|
| `main` | Parse argv, wire layers, run event loop, write file on exit | All |
| `FileDetector` | Infer git context from file path (COMMIT_EDITMSG, MERGE_MSG, git-rebase-todo) | FileParser, App |
| `FileParser` | Read raw file bytes → Document (editable lines + read-only comment lines) | App (initial state) |
| `GitContext` | Enum capturing which git operation is in progress; carries parsed metadata | App, Renderer |
| `Document` | Holds editable text as Vec of lines, cursor position, comment regions | App, FileWriter, Renderer |
| `App` | Owns all runtime state; applies Actions to Document; decides mode transitions | InputHandler, Renderer |
| `InputHandler` | Translates crossterm KeyEvents → typed Action enum | App |
| `Renderer` | Reads App state, calls `terminal.draw()` to paint widgets each frame | App (read-only) |
| `FileWriter` | Serializes Document back to disk, preserves comment lines exactly | App (on save) |

## Recommended Project Structure

```
src/
├── main.rs              # argv, wires all layers, event loop, exit code
├── context.rs           # GitContext enum + FileDetector logic
├── parser.rs            # FileParser: raw bytes → Document
├── document.rs          # Document struct (lines, cursor, comments)
├── app.rs               # App struct, Action enum, state transitions
├── input.rs             # InputHandler: KeyEvent → Action
├── renderer.rs          # Renderer: ratatui widgets, layout, styles
└── writer.rs            # FileWriter: Document → disk
```

### Structure Rationale

- **context.rs + parser.rs separate:** Detection (what git operation?) and parsing (split editable vs comment content) are different concerns. Detection can be tested with path strings alone; parsing needs file content.
- **document.rs standalone:** The in-memory text model has no UI or git dependency. Keeping it isolated enables unit testing cursor movement and line operations without a terminal.
- **app.rs as single state owner:** All mutable runtime state lives here. Renderer and InputHandler receive read/write references to App, never to each other. This mirrors the Elm architecture's single source of truth without framework overhead.
- **renderer.rs read-only:** Renderer only reads App state; it never mutates it. This prevents rendering code from accidentally triggering state changes.

## Architectural Patterns

### Pattern 1: Immediate-Mode Rendering (Ratatui standard)

**What:** Every tick, the entire UI is re-drawn from current App state. No retained widget tree. `terminal.draw(|frame| renderer.render(frame, &app))` is called on every render tick.

**When to use:** Always — this is ratatui's model. Especially appropriate for a tool this small; no diffing overhead needed.

**Trade-offs:** Simple mental model (state → pixels, no sync issues). Slightly more CPU than retained mode, irrelevant at <60fps for a text editor.

```rust
loop {
    terminal.draw(|frame| renderer.render(frame, &app))?;
    if let Some(event) = crossterm::event::read()? {
        let action = input_handler.handle(event);
        match app.apply(action) {
            Outcome::Save => { writer.write(&app.document)?; break Ok(0); }
            Outcome::Cancel => break Ok(1),
            Outcome::Continue => {}
        }
    }
}
```

### Pattern 2: Action Enum as Event Bus

**What:** InputHandler converts raw key events into a typed `Action` enum (`EditChar(char)`, `MoveCursor(Direction)`, `Save`, `Cancel`, `ToggleHelp`). App's `apply()` method matches on Action.

**When to use:** Any TUI with more than a handful of key bindings. Prevents key-handling logic from leaking into App.

**Trade-offs:** Small extra indirection. Major benefit: input can be tested by constructing Actions directly, without faking terminal events.

```rust
enum Action {
    InsertChar(char),
    Backspace,
    MoveCursor(CursorDir),
    NewLine,
    Save,        // Ctrl+S
    Cancel,      // Esc
    ToggleHelp,
}
```

### Pattern 3: Document as Typed Text Model (not raw String)

**What:** Instead of storing the file as a single `String`, represent it as:
- `editable_lines: Vec<String>` — what the user can modify
- `comment_lines: Vec<(usize, String)>` — line index + content, rendered as styled/non-editable
- `cursor: (row, col)` — current insertion point

**When to use:** Whenever comments must be visually distinct and preserved byte-for-byte on save. Critical for git files where `# ` comment lines must not be altered.

**Trade-offs:** Slightly more complex serialization (must reassemble original interleaving on write). Worth it to prevent accidental comment corruption.

## Data Flow

### Startup Flow

```
git invokes gitmedit <file_path>
        ↓
main.rs: read argv[1]
        ↓
FileDetector: path → GitContext (Commit | Merge | Rebase | Squash hint)
        ↓
FileParser: read file bytes → Document (editable lines + comment regions)
        ↓
App::new(document, context) — initial state ready
        ↓
terminal setup (crossterm raw mode, alternate screen)
        ↓
event loop begins
```

### Edit Cycle (per keypress)

```
crossterm::event::read() → KeyEvent
        ↓
InputHandler::handle() → Action
        ↓
App::apply(action) → mutates Document, returns Outcome
        ↓
terminal.draw(|f| renderer.render(f, &app))
        ↓
(loop until Outcome::Save or Outcome::Cancel)
```

### Save Flow

```
Action::Save received
        ↓
App validates: is Document empty? → warn if yes, require second Ctrl+S or continue
        ↓
Outcome::Save returned to main
        ↓
FileWriter::write(&document, &path) — serialize editable + comment lines in original order
        ↓
terminal teardown (restore terminal state)
        ↓
process::exit(0)  ← git reads this as "use the message"
```

### Cancel Flow

```
Action::Cancel received
        ↓
terminal teardown
        ↓
process::exit(1)  ← git reads this as "abort the operation"
```

### Key Data Flows Summary

1. **git → editor:** Single file path in argv[1]. File content is the message to edit.
2. **editor → git:** Exit code 0 (commit proceeds with file content as-is) or 1 (abort). File is written before exit 0.
3. **Context detection → Renderer:** GitContext informs which UI layout to show (e.g., rebase mode shows todo-list view, not a text area).
4. **Document → FileWriter:** Editable lines and comment lines are reassembled into original byte order. No line endings are changed unnecessarily.

## Git Context Detection

git passes a single file path. The filename determines context:

| Filename | GitContext | UI Mode |
|----------|------------|---------|
| `COMMIT_EDITMSG` | `Commit` | Single text area, type the message |
| `MERGE_MSG` | `Merge` | Text area with merge conflict metadata visible as comments |
| `git-rebase-todo` | `Rebase` | Todo-list editor: pick/squash/drop lines are structured rows |
| `TAG_EDITMSG` | `Tag` | Same as Commit; minimal differences |

Detection is done by `Path::file_name()` match — no content inspection needed for type detection (content inspection is for parsing).

Squash context is inferred from Rebase: when all non-comment lines use `squash` or `fixup` verbs, or a `# This is a combination of N commits` comment is present at the top of COMMIT_EDITMSG.

## Anti-Patterns

### Anti-Pattern 1: Treating the File as a Plain String

**What people do:** Read the entire file into one `String`, let the user edit it, write it back.

**Why it's wrong:** Comment lines (lines starting with `#`) get modified or re-flowed. Git uses comment content for the "verbose" diff preview and for rebase instructions. Corrupting them causes silent failures or confusing git behavior.

**Do this instead:** Split lines on parse, tag each as editable or comment, reassemble in original order on write.

### Anti-Pattern 2: Blocking the Event Loop on File I/O

**What people do:** Call `std::fs::read_to_string` synchronously inside the render/event loop.

**Why it's wrong:** The git message files are tiny (typically <100 lines) so this is harmless in practice, but if the pattern extends to reading `.git/COMMIT_EDITMSG` history or diff stats it blocks raw-mode terminal, causing visual glitches.

**Do this instead:** Do all file I/O before entering raw mode and after exiting it. The event loop only reads from and writes to the in-memory Document.

### Anti-Pattern 3: One Monolithic App Struct with Rendering Logic

**What people do:** Put `terminal.draw(...)` calls and widget construction inside `App::update()`.

**Why it's wrong:** Makes state logic untestable without a terminal. Mixing concerns causes widget layout bugs to look like state bugs.

**Do this instead:** Keep App purely concerned with state. Renderer is a separate struct that receives `&App` as read-only input.

### Anti-Pattern 4: Ignoring Exit Codes

**What people do:** Always exit 0 (success), even on Cancel.

**Why it's wrong:** Git uses the editor's exit code to decide whether to proceed. Exit 0 on cancel causes git to commit an empty or stale message silently.

**Do this instead:** Map Cancel/empty-abort → `process::exit(1)`. Map Save → write file, then `process::exit(0)`.

## Integration Points

### External Interfaces

| Interface | Direction | Notes |
|-----------|-----------|-------|
| `argv[1]` (file path) | git → gitmedit | Only input from git. Path is absolute. |
| File write at argv[1] | gitmedit → git | Write edited content before exit 0. |
| Exit code 0/1 | gitmedit → git | 0 = success, 1 = abort. Any other non-zero also aborts. |
| `$VISUAL` / `$EDITOR` | Optional fallback | Not needed since gitmedit is set explicitly via `core.editor` |
| `GIT_EDITOR_SEQUENCE` | git → gitmedit | Not applicable (sequence editor is separate from commit editor) |

### Internal Module Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `context` ↔ `parser` | `GitContext` passed to `FileParser::parse()` | Context informs how to classify lines (e.g., rebase todo verbs vs free-text) |
| `parser` → `app` | `Document` (owned, moved in) | One-time transfer at startup |
| `app` ↔ `renderer` | `&App` (read-only borrow each frame) | Renderer never mutates |
| `input` → `app` | `Action` enum (value type) | No shared state; input produces values, app consumes them |
| `app` → `writer` | `&Document` + `&Path` (on save) | Writer is stateless; called once |

## Build Order Implications

Dependencies flow upward; build and test in this order:

1. **`document.rs`** — no dependencies. Cursor ops, line splits, comment flagging. Can be fully unit-tested standalone.
2. **`context.rs` + `parser.rs`** — depend only on `document.rs`. Test with fixture files (COMMIT_EDITMSG samples, rebase-todo samples).
3. **`writer.rs`** — depends on `document.rs`. Test round-trip: parse → write → compare bytes.
4. **`app.rs`** — depends on `document.rs` and `context.rs`. Test state transitions with Actions directly (no terminal needed).
5. **`input.rs`** — depends on `app.rs` Action enum. Test KeyEvent → Action mapping.
6. **`renderer.rs`** — depends on `app.rs`. Integration-test with ratatui's TestBackend.
7. **`main.rs`** — wires everything. Integration test via subprocess: spawn process with a fixture file, send keys, check exit code and file output.

## Scaling Considerations

This is a single-user, single-session CLI tool. "Scaling" means session startup latency and correctness across edge cases.

| Concern | Approach |
|---------|----------|
| Startup time | All parsing is synchronous and in-memory; git files are <100 lines. Target <50ms cold start. |
| Large rebase todo files | Scrollable viewport on Document; render only visible lines. Even 500-entry rebases are trivial. |
| Unicode in commit messages | Use `unicode-width` crate for display-width vs byte-length. Cursor math must use character indices, not byte indices. |
| Terminal resize | Handle `crossterm::event::Event::Resize` in event loop; re-render at new dimensions. |
| Alternate screen cleanup | Always restore terminal on panic via `Drop` or `std::panic::set_hook`. |

## Sources

- [Ratatui Component Architecture](https://ratatui.rs/concepts/application-patterns/component-architecture/) — component trait pattern, Action-based communication
- [Ratatui Elm Architecture](https://ratatui.rs/concepts/application-patterns/the-elm-architecture/) — model/update/view separation rationale
- [Ratatui Rendering Concepts](https://ratatui.rs/concepts/rendering/) — immediate-mode rendering loop
- [tui-textarea crate](https://github.com/rhysd/tui-textarea) — TextArea struct pattern: state + Widget trait + input() method
- [gitui source structure](https://github.com/gitui-org/gitui) — asyncgit separation, modular UI components
- [git-interactive-rebase-tool](https://github.com/MitMaro/git-interactive-rebase-tool) — dedicated rebase sequence editor as reference for todo-file handling
- [Git rebase documentation](https://git-scm.com/docs/git-rebase) — rebase-todo file format, command verbs

---
*Architecture research for: TUI git editor (gitmedit)*
*Researched: 2026-03-18*
