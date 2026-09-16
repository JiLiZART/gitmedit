## Context

See proposal.md — Why. The current editor is three large files: `app.rs` (state), `renderer.rs`
(drawing), and `document.rs` (line classification, protected-line serialization, rebase parsing).
Line protection by mode, the squash header split, and horizontal scrolling all live there and are
all being removed. `terminal.rs` enters the alternate screen and enables mouse capture already, but
its `Drop` calls `.unwrap()` and its panic hook only disables raw mode.

Constraints that shape the design (from `openspec/config.yaml`): byte-exact round trips, first frame
under 100 ms, no git library, terminal always restored. Verified facts the design relies on:

- `ratatui-textarea` 0.8 has no soft wrap, but exposes `lines()`, `cursor()`, `selection_range()`,
  and `move_cursor(CursorMove::Jump(row, col))`.
- crossterm 0.29 delivers `ESC b` / `ESC f` as `Char('b')` / `Char('f')` with the ALT modifier.
- git 2.51 runs the message editor on `<gitdir>/COMMIT_EDITMSG` during a reword. Its
  `rebase-merge/done` ends with `reword <full 40-char sha> # subject`, while the todo handed to the
  sequence editor uses abbreviated hashes.

## Goals / Non-Goals

**Goals:**
- Every parsing and file rule lives in a small, pure, unit-tested module with fixture tests.
- The crate compiles and all tests pass after every task.
- One state machine owns all UI state, and the renderer and key mapping are thin.

**Non-Goals:**
- Actions inside the status pane beyond scrolling (no diff preview, collapse, or path insertion).
- Display-cell-accurate width for wide CJK/emoji characters.
- Standalone commit mode and cross-platform verification (separate changes).

## Decisions

**New modules beside the old ones, one switch at the end.** New modules are added one at a time:
`wrap`, `message`, `status`, `rebase`, `reword`, `details`, `layout`, `session`, `keys`, `ui`.
`main.rs` moves to them in a single task that deletes `app.rs`, `renderer.rs`, and `document.rs`.
Alternative: rewrite `app.rs` in place. Rejected because the binary would not compile for several
tasks, and neither would the tests, since every test module is compiled into the one bin crate.

**Soft wrap is drawn by the renderer; TextArea stays the editing engine.** TextArea keeps doing input,
undo/redo, selection, and yank. `wrap::wrap(line, width)` returns segments with char offsets, and
`wrap::locate` maps the cursor column to a display row. The renderer draws wrapped rows itself, and
Up/Down are intercepted to jump by display row. Alternative: switch to an editor widget with wrapping.
Rejected because it adds a dependency and a rewrite of the editing keys.

**Comments leave the editor through a message/trailer split.** The trailer starts at the first
comment line and is written back verbatim after exactly one blank line. Git's own files always have
that shape, so they round-trip byte for byte. Non-comment lines found inside the trailer (squash files)
move into the message. The trade-off: such a file is reordered on save but strips to the same commit
message. Alternative: re-insert comment lines at their original positions after editing. Rejected
because positions become meaningless once the user adds or deletes lines.

**Todo lines keep their raw text.** Each commit line stores its raw text plus the action and flag it
was parsed with. Serialization emits the raw text unless the action or flag changed, which gives
byte-exact round trips and keeps abbreviations on untouched lines. Reordering swaps whole lines among
instruction slots, so comments stay put.

**Pending rewords are kept outside the todo model.** `rewords: hash → new subject` is held in session
state. The todo line keeps its original subject text; only the action word changes.
- **On save:** stored messages are replaced wholesale in `<gitdir>/gitmedit/reword/<hash>`. Each
  holds the new subject plus the original body, read with `git log -1 --format=%B`. Only hex-hash
  names are written, so a hand-edited todo cannot write outside that directory.
- **On opening a todo:** stored messages matching a reword line are loaded back into state, and all
  others are deleted, which also cleans up after abandoned rebases.
- **During the reword:** when git opens `COMMIT_EDITMSG`, the last `done` line is matched by hash
  prefix in either direction. The stored message replaces the editor's message, and the file is
  deleted after a successful save.

Alternative: rewrite the todo as `pick` plus `exec git commit --amend -m`. Rejected in brainstorming
because it adds exec lines and a second rewriting path.

**The renderer takes `&mut App` and writes back viewport state.** Scroll offsets are clamped against
content height at draw time, and the renderer stores pane rectangles (for mouse hit tests), editor
wrap width (for Up/Down), and page height (for PgUp/PgDn). End sets `usize::MAX`, and the next draw
clamps it. Alternative: compute layout in `main` before drawing, or use `Cell` fields. Both duplicate
layout math or hide mutation.

**Focus doubles as narrow-mode visibility.** Below 100 columns the visible pane is the focused pane,
so Ctrl+T, Alt+arrows, Esc, and resizing all follow from one field. There is no separate "show right"
flag to keep in sync.

**Commit details load on a background thread.** A worker thread receives hashes over an mpsc channel
and runs `git --git-dir <gitdir> show --no-color --format=%B%x00 --name-status <hash>` with stdin
closed. It sends back `Option<CommitDetails>`, which is cached by hash. The event loop polls input with
a 100 ms timeout, drains results, and redraws. ratatui's buffer diff keeps idle redraws cheap. Nothing
git-related blocks the first frame except `core.commentChar`, as before.

**Alternate screen and mouse capture, replacing the inline-rendering goal.** Two full-height panes and
click-to-focus need both. The retired `restore-inline-rendering` change is superseded. Its guard fix
(no panic in `Drop`) is kept.

## Risks / Trade-offs

- [Mouse capture disables the terminal's own text selection] → Users hold Shift (Option in iTerm2)
  to select. Copy and paste within the message uses Ctrl+C/Ctrl+V. The README will say so.
- [Wrap counts chars, not cells; wide characters can overflow a row] → Accepted and marked with a
  `ponytail:` comment in `wrap.rs`. Upgrade to `unicode-width`, already a transitive dependency of
  ratatui, if reported.
- [A cursor at the end of a completely full row is drawn on the last column] → Accepted, with a
  comment at the cursor placement.
- [Interleaved squash files are not byte-identical after save] → Git's default `strip` cleanup yields
  the same message. Covered by a test asserting the non-comment content.
- [Terminals that swallow Alt+arrows] → `ESC b`/`ESC f` fallbacks and click-to-focus. Tracked for real
  terminals in `cross-platform-verification`.
- [Inline reword silently degrades when `core.editor` is another program] → Git opens that editor with
  the original message, which is still correct. Help overlay and README state the prerequisite.
- [Holding Down queues one `git show` per commit on a sequential worker] → The table stays responsive
  and details catch up. Accepted for typical todo sizes.
- [Stored reword files survive a crash between todo save and the reword stop] → They are deleted the
  next time any todo opens, or when unmatched.

## Migration Plan

All work lands on `feature/v1.0.0` and ships as version 1.0.0; existing git configuration keeps
working with no user action. Rollback is reverting the branch merge. `/opsx:archive` merges the delta
specs into `openspec/specs/`, which retires `merge-mode` and `squash-mode`.
