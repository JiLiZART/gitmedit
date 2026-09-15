Conventions for every task:
- Write the failing test in the file's `#[cfg(test)] mod tests` first, then implement.
- Fixtures load with `include_str!("../fixtures/<name>.txt")`.
- Each group ends with a conventional commit, with no AI attribution lines.
- New modules are added to the `mod` list in `src/main.rs` when created. `dead_code` warnings are
  expected until group 14.

## 1. Terminal restore (`src/terminal.rs`, `tests/terminal_integration.rs`)

- [x] 1.1 Add `write_restore_sequences(out: &mut impl Write)` (disable mouse capture, leave alternate screen, show cursor — each step runs, errors ignored) and `restore_terminal()` (sequences on stdout, then disable raw mode); verify a `#[cfg(unix)]` test finds `\x1b[?1000l` before `\x1b[?1049l` before `\x1b[?25h` in a `Vec<u8>`, and a writer that always errors does not panic
- [x] 1.2 Make the panic hook and `TerminalGuard::drop` call `restore_terminal()` (drop flushes the backend first; no `unwrap`), and have `TerminalGuard::new` restore before returning a setup error; verify `cargo test terminal::tests` passes and `grep unwrap src/terminal.rs` finds nothing outside tests
- [x] 1.3 Rewrite the ignored PTY test as `enters_alternate_screen_and_restores_terminal_on_exit`, asserting the output contains `\x1b[?1049h`, `\x1b[?1049l`, and `\x1b[?1000l`; verify `cargo test --test terminal_integration -- --ignored` passes

## 2. Context detection (`src/context.rs`)

- [x] 2.1 Make `GitContext` `Copy`, add `Tag` for `TAG_EDITMSG`, add `is_message(self)` (commit, merge, squash, tag); verify tests for every filename, full paths, and `is_message`
- [x] 2.2 Add `git_dir(path, context) -> Option<PathBuf>` (parent for message files, parent of `rebase-merge` for the todo, `None` for unknown); verify `/repo/.git/worktrees/feature/COMMIT_EDITMSG` → `/repo/.git/worktrees/feature` and `/repo/.git/rebase-merge/git-rebase-todo` → `/repo/.git`
- [x] 2.3 Add `read_comment_char()` (git config with stdin closed) and pure `parse_comment_char(&str)`; leave `document::read_comment_char` untouched until group 14; verify `";\n"` → `;`, `""` → `#`, `"auto\n"` → `#`

## 3. Wrapping (`src/wrap.rs`)

- [x] 3.1 Add `Segment { start, text }` and `wrap(line, width) -> Vec<Segment>`:
  - break at the last space within width, consuming the space
  - hard-break words wider than width
  - width 0 or a line that fits gives one segment
  - add a `ponytail:` comment that widths are char counts, not terminal cells

  Verify: `wrap("hello world foo bar baz", 11)` gives `["hello world", "foo bar baz"]` with the second segment starting at char 12; `wrap("abcdefghijklmnop", 5)` gives `abcde/fghij/klmno/p`; empty input gives one empty segment; rejoining the rows loses no text.
- [x] 3.2 Add `locate(segments, col) -> (row, col_in_row)`; verify for `wrap("hello world foo", 10)` that col 0→(0,0), 5→(0,5), 6→(1,0), 15→(1,9)

## 4. Message split (`src/message.rs`)

- [x] 4.1 Add `is_scissors(line, cc)` and `split(raw, cc) -> MessageFile { message, trailer, final_newline }`:
  - the trailer runs from the first comment line (above any scissors line) to the end of the file
  - the message is the lines before it, with trailing blank lines trimmed
  - non-blank, non-comment lines inside the trailer move into the message
  - a blank line is inserted between moved runs that were separated in the file

  Verify:
  - `ammend2` → message `["fix: release volume"]`
  - `merge_fixture` → one message line, and the blank lines between comment groups stay in the trailer
  - `"\n# Please…"` → empty message
  - an interleaved squash file → `["feat: one","","body one","","fix: two"]`
  - the scissors line and diff lines stay in the trailer
- [x] 4.2 Add `MessageFile::assemble(&self, message)`: trimmed message, one blank line and the trailer when there is a trailer, then the final newline if the original had one. Verify:
  - `ammend`, `ammend2`, `merge`, `merge2`, `rebase_fixture` and `pull_rebase_fixture` all round-trip byte for byte
  - an edited message keeps the trailer
  - trailing blanks in the edited message collapse to one separator
  - a file without comments has no trailer
  - a missing final newline is preserved
  - a custom `;` comment char works

## 5. Status pane model and rendering (`src/status.rs`)

- [x] 5.1 Add the types and headings:
  - `SectionKind`, in this order: Branch, Merge, Rebase, Squash, Conflicts, Staged, Unstaged, SubmodulesStaged, SubmodulesNotUpdated, Untracked, Other, Diff. It has `title()` and `counted()`.
  - `Entry::{File{badge,path,suffix}, Submodule, Commit, Warning, Text, DiffLine}`.
  - `Section { kind, entries }` with `count()`, which counts File and Submodule entries.
  - `Status { branch, sections }` with `is_empty()` and `section(kind)`.
  - `parse(trailer, cc)` recognizes headings (including `Unmerged paths:`), skips boilerplate (the "Please enter…" lines, `with '…`, `(…)` hints, and the merge `update-ref` hint lines), sends merge/rebase/squash state lines to their sections, puts entries in the current list section, and sends everything else to Other verbatim. Sections are sorted by kind.

  Verify these fixture counts:
  - `ammend_fixture`: Staged 34.
  - `merge_fixture2`: kinds `[Branch, Merge, Conflicts, Staged, Unstaged, SubmodulesStaged, SubmodulesNotUpdated]`; Conflicts 4, Staged 160, Unstaged 1, each submodule section 1.
  - `rebase_fixture`: kinds `[Branch, Rebase, Conflicts, Staged, Unstaged, SubmodulesStaged, SubmodulesNotUpdated, Untracked]`; Rebase entry 0 is the "interactive rebase in progress; onto c3d42a1f4" text and entry 2 is the commit `pick c0ad61ef0 # Update chat message stop words`; Unstaged 6; not-updated 5 with 4 warnings; Untracked `?` `vendor/money-tree/`.
  - `pull_rebase_fixture`: only Conflicts, 6 `U` entries.
- [x] 5.2 Build the compact branch summary from `On branch`/`HEAD detached`, ahead/behind/diverged (`and have A and B …`)/up-to-date, plus `Date:`/`Author:` extras; set `Status.branch`. Verify:
  - `ammend_fixture` → `EX-3211-arc-epic → origin/EX-3211-arc-epic  ahead 1, behind 2`, then `Date: Tue May 12 01:43:54 2026 +0200`
  - `ammend2` → `TASK-1111-fix-stage-view → origin/TASK-1111-fix-stage-view  ahead 1`
- [x] 5.3 Parse file entries: status word → badge (modified M, new file A, deleted D, renamed R, copied C, typechange T, unmerged states U); bare path → `U`, or `?` in Untracked; a trailing ` (…)` becomes the suffix. Submodule lines: `* ` → Submodule, `> `/`< ` → Commit, `Warn:` → Warning. After a scissors line, non-comment lines become DiffLine. Verify:
  - badges M/A/D/R/U, with the rename path `old.rs -> new.rs`
  - `packages/@dev-kit (new commits)` has suffix `(new commits)`
  - a heading with no entries has count 0
  - `# something unexpected` goes to Other
  - diff lines are collected
  - a `;` comment char works
  - an empty trailer gives an empty status
- [x] 5.4 Add `badge_style(char)` (A green, D red, M yellow, U red bold, R/C/T blue, other dim) and `render_lines(&Status, width)`:
  - a cyan bold header with the count for counted sections
  - a blank line between sections
  - file rows ` B path`, wrapped at width−3 with a 3-space continuation indent, suffix dimmed on the last row
  - Text/Commit/Warning rows wrapped with an indent
  - diff `+` green, `-` red, `@@` cyan

  Verify the rendered rows `Branch`, ` main`, ``, `Staged (2)`, ` M packages/@dev-kit (new commits)`; the `D` span is red; width 12 wraps into `[" M aaaa/bbbb", "   /cccc/ddd", "   d"]`; the diff colors are right.

## 6. Rebase todo model (`src/rebase.rs`)

- [ ] 6.1 Add `Action::{Pick, Reword, Edit, Squash, Fixup, Drop}` with `parse` (long and one-letter), `from_key` (p r e s f d), `as_str`, `cycled` (pick→squash→fixup→drop→pick; reword/edit→squash); verify key and cycle tests
- [ ] 6.2 Parse lines and serialize:
  - `CommitLine { action, flag, hash, rest }` keeps the raw text and the original action/flag, and has `subject()`, which strips `# `.
  - `TodoLine::{Commit, Other, Comment, Blank, Unknown}` with `is_instruction()`.
  - Other = exec/x, break/b, label/l, reset/t, merge/m, update-ref/u, noop.
  - `fixup -C|-c` is parsed as a flag.
  - `Todo { lines, final_newline }`; `parse(raw, cc)` and `serialize()` emit raw text for unchanged lines.

  Verify:
  - `squash_fixture` round-trips byte for byte
  - `p abc1234 Fix bug` keeps `p`
  - `fixup -C 7314ba6 subject` round-trips
  - exec and break are Other, garbage is Unknown
  - `""`, `"\n"` and input without a final newline round-trip
- [ ] 6.3 `CommitLine::set_action` clears the flag unless the new action is fixup; changed lines serialize as `action [flag] hash rest`. Add `Todo::range(cc)` from the `Rebase a..b onto` comment. Verify:
  - changing line 1 to fixup makes exactly one line differ: `fixup 45f8bcd # docs(08): create phase plan`
  - `fixup -c` → pick drops the flag
  - `range` is `36d7eda..aa619f8`
- [ ] 6.4 Add `instruction_indices()`, `commit(line)`, `commit_mut(line)`, and `move_instruction(line, up) -> Option<new_line>`, which swaps with the neighbouring instruction. Verify:
  - `pick A/# comment/pick B`: moving B up gives `pick B/# comment/pick A`
  - moving past either end returns `None`
  - 40 arbitrary moves keep the multiset of hashes
- [ ] 6.5 Add `Summary { total, result, squash_fixup, drop, reword, first_is_squash }` and `summary()`, where pick/reword/edit count as results; verify the fixture with lines 1–2 squash, 3 drop and 4 reword gives (14, 11, 2, 1, 1), and a leading fixup sets `first_is_squash`

## 7. Reword storage (`src/reword.rs`)

- [ ] 7.1 Add `store_dir(git_dir)` = `<gitdir>/gitmedit/reword` and the storage functions:
  - `replace_subject(original, subject)`: new subject plus the original body, or `subject\n` when there is no body.
  - `store(git_dir, &HashMap<hash, subject>, full_message)`: remove the directory, write one file per pending reword, and skip names that aren't hex.
  - `git_full_message(git_dir, hash)`: `git --git-dir <dir> log -1 --format=%B <hash>` with stdin closed.

  Verify with tempdir tests:
  - `replace_subject(Some("c3\n\nbody 3\n"), "new")` returns `new\n\nbody 3\n`
  - an empty map removes the directory
  - a non-hex hash writes nothing
- [ ] 7.2 Add `load(git_dir, reword_hashes: &[&str]) -> HashMap<hash, subject>`, matching stored names to todo hashes by prefix in either direction, keeping the first line of each match, and deleting unmatched files; verify a stored `54763e6` loads for a reword line and a stale `deadbeef` is deleted
- [ ] 7.3 Add `pending_for_commit(git_dir) -> Option<(PathBuf, String)>` from the last non-empty line of `<gitdir>/rebase-merge/done` (`reword`/`r` plus a full sha matched against stored names); verify `reword 545ca5d95e7b6bbb297681be181a60d396ee8ee8 # c3` finds stored `545ca5d`, while a pick line or a missing done file returns `None`

## 8. Commit details (`src/details.rs`)

- [ ] 8.1 Add `CommitDetails { message: Vec<String>, files: Vec<(char, String)> }` and `parse_show(out)`: split at NUL; message is the trimmed lines; each name-status line becomes (first status char, tab-separated paths joined by ` -> `); verify unit tests for add, modify and `R100\told\tnew`
- [ ] 8.2 Add `DetailsLoader::spawn(git_dir)` with a worker thread over mpsc running `git [--git-dir d] show --no-color --format=%B%x00 --name-status <hash>` with stdin null, plus `request(hash)` (deduplicated), `poll() -> bool` (drain into the cache) and `get(hash) -> Option<&Option<CommitDetails>>`. Verify an integration test that:
  - creates a temp repo, committing with `-c commit.gpgsign=false -c user.email=t@t -c user.name=t`
  - checks HEAD gives message `["subject","","body"]` and files `[('A',"a.txt")]` within 5 s
  - checks an unknown hash gives `Some(None)`
  - returns early if git can't be run

## 9. Layout (`src/layout.rs`)

- [ ] 9.1 Add `NARROW_WIDTH = 100`, `Pane::{Left, Right}` and `PaneRects { left, right, key_bar }`, plus:
  - `compute(area, has_right, focus)`: key bar on the last row; no right pane → left takes the body; narrower than 100 → only the focused pane, full width; otherwise a 60/40 split.
  - `pane_at(rects, col, row)`.

  Verify:
  - 120×30 gives left width 72, right width 48, key bar at y=29
  - 80 columns with focus Right gives right only, width 80
  - 80 columns with focus Left gives left only
  - hit tests work on both panes

## 10. Session: message and plain layouts (`src/session.rs`)

- [ ] 10.1 Add the state types and constructor:
  - `ScrollBy::{Lines, Pages, Top, End}`, `Action` (Save, Cancel, ToggleHelp, ScrollHelp, FocusLeft, FocusRight, ToggleFocus, ClickAt, WheelAt, ScrollRight, Edit(Input), CursorUp, CursorDown, DeleteLine, Undo, Redo, DeleteWord, DeleteNextWord, Copy, Cut, Paste), `Outcome::{Save, Cancel, Continue}`.
  - `Body::{Message(MessageBody), Plain(TextArea), Rebase(RebaseBody)}`.
  - `App` has public fields: body, context, git_dir, comment_char, file_name, focus, show_help, help_scroll, right_scroll, and renderer-written rects, editor_width, right_page, editor_top, table_top.
  - `App::new(raw, context, git_dir, comment_char)`: message contexts split the file and parse status, and commit prefills from `reword::pending_for_commit`; rebase builds a `RebaseBody` with `DetailsLoader`, requests details for the selection, and loads stored rewords; unknown files get a plain editor.
  - `has_right()` is true for rebase, for message with a non-empty status, and false for plain.

  Verify:
  - `ammend2`: the editor lines are `["fix: release volume"]`, `has_right` is true, and `serialized_content()` equals the raw file
  - an unknown file keeps its comment lines and `has_right` is false
  - `"just text\n"` as a commit has `has_right` false
- [ ] 10.2 Add the non-editing actions:
  - `apply(Action)`: FocusRight only when `has_right`; ToggleFocus; ClickAt via `layout::pane_at`; ToggleHelp resets `help_scroll`; ScrollHelp.
  - `ScrollRight`: Lines/Pages use `right_page`, Top = 0, End = `usize::MAX`.
  - `WheelAt`: scrolls the right pane or moves the left cursor by display row.

  Verify focus, click with manually computed rects, End → `usize::MAX`, and that Lines(-1) from 5 gives 4.
- [ ] 10.3 Route editing actions to `active_editor_mut()` (message, plain, inline or raw editor):
  - DeleteLine = Head then `delete_line_by_end`
  - clipboard through `arboard`, doing nothing when no clipboard is available
  - CursorUp/Down move by display row using `wrap` with `editor_width`, via `CursorMove::Jump`

  Verify with `editor_width = 10` and lines `["hello world foo", "x"]`: CursorDown from (0,0) → (0,6) → (1,0), and CursorUp → (0,6).
- [ ] 10.4 Add `serialized_content()` (message assemble; plain lines joined plus `\n`; rebase `todo.serialize()`) and `finish_save()` (removes a used reword file; rebase calls `reword::store` with `git_full_message`). Verify a tempdir with `rebase-merge/done` ending `reword 545ca5d…` and a stored `545ca5d` of `new subject\n\nbody 3\n`:
  - `App::new` on the commit message makes the editor lines `["new subject","","body 3"]`
  - the serialized content keeps the trailer
  - `finish_save` removes the stored file

## 11. Session: rebase actions (`src/session.rs`)

- [ ] 11.1 Add `SelectBy(ScrollBy)`: Lines, Pages ×10, Top and End, clamped. Selection changes request details and reset `right_scroll`, and the wheel over the table moves the selection. Verify on `squash_fixture` that Down/Up/End/Top land on the expected index and that selection stops at the ends.
- [ ] 11.2 Add `SetAction(rebase::Action)` and `CycleAction`, where any action other than Reword removes a pending reword, and `MoveInstruction { up }`, where the selection follows the moved row. Verify:
  - `SetAction(Fixup)` on the first row serializes as `fixup 54763e6 # docs(state): record phase 8 context session`
  - Tab turns pick into squash
  - moving up at index 0 does nothing
  - moving down sets the selection to 1 and swaps the lines
- [ ] 11.3 Add inline reword:
  - `StartInline` opens a one-line `TextArea` holding the display subject, cursor at the end.
  - `CommitInline` trims the text. Empty, or equal to the original subject, removes the pending reword; otherwise it sets Reword and stores `rewords[hash]`.
  - `CancelInline`; `RebaseBody::display_subject(commit)`.

  Verify:
  - editing to `new subject` gives `rewords["54763e6"] == "new subject"`
  - that line serializes as `reword 54763e6 # docs(state): record phase 8 context session`
  - Cancel leaves no reword
  - `SetAction(Pick)` afterwards clears the reword
- [ ] 11.4 Add `ToggleRaw` and save behaviour:
  - Entering raw mode opens a `TextArea` of `todo.serialize().lines()` and closes any inline edit.
  - Leaving joins the lines (plus the final newline), re-parses, keeps rewords only for hashes still marked reword, clamps the selection, and requests details.
  - `Save` commits an open inline edit and leaves raw mode before returning `Outcome::Save`.

  Verify:
  - toggling twice without edits serializes to the fixture
  - inserting `exec cargo test` after the last pick in raw mode gives 15 instructions, the last one `Other`
  - `Save` with an inline edit open stores the reword
- [ ] 11.5 Add `poll_background() -> bool` (drains details). Verify with a tempdir git dir:
  - a stored subject is loaded for a todo whose first line is `reword 54763e6 …`
  - a stale stored file is deleted
  - after a reword, `finish_save` writes `gitmedit/reword/54763e6` containing `new subject\n` (git lookup fails in the tempdir)

## 12. Key mapping (`src/keys.rs`)

- [ ] 12.1 Map mouse input in `map_event(&Event, &App) -> Option<Action>`:
  - Left-button down → `ClickAt`, unless help is visible.
  - Wheel up/down → `WheelAt(±1)`, or `ScrollHelp(±1)` while help is visible.
  - Only key Press events are handled.

  Verify that click and wheel events map correctly and that a key Release event maps to `None`.
- [ ] 12.2 Apply key precedence, top rule first:
  1. Help visible: Esc and Ctrl+H map to ToggleHelp, Up/Down map to ScrollHelp, everything else is `None`.
  2. Global: Ctrl+S → Save, Ctrl+H → ToggleHelp.
  3. Inline editor open: Enter → CommitInline, Esc → CancelInline, everything else is an editing key.

  Verify: `a` with help open gives `None`; Esc with help open toggles it; Alt+Right while inline editing gives `Edit`.
- [ ] 12.3 Add focus keys: Alt+Left/`Alt+b` → FocusLeft, Alt+Right/`Alt+f` → FocusRight, Ctrl+T → ToggleFocus. With the right pane focused: Esc → FocusLeft, Up/Down/PgUp/PgDn/Home/End → `ScrollRight`, anything else `None`. With the left pane focused, Esc → Cancel. Verify all of these.
- [ ] 12.4 Add table and editor keys:
  - Table: Alt+Up/Down → MoveInstruction; Up/Down/PgUp/PgDn/Home/End → SelectBy; Tab → CycleAction; Enter → StartInline; Ctrl+E → ToggleRaw; unmodified p r e s f d → SetAction.
  - Raw mode: Ctrl+E → ToggleRaw.
  - Editors: unmodified Up/Down → CursorUp/CursorDown; Ctrl+U/Z/Y/W/D/C/X/V → their actions; anything else → `Edit(Input::from(key))`.

  Verify: `f` → SetAction(Fixup), Tab, Alt+Up, Enter and Ctrl+E map correctly, and `a` in a message maps to `Edit`.

## 13. Rendering (`src/ui.rs`)

- [ ] 13.1 `render(frame, &mut App)` computes the layout, stores `rects`, and draws bordered panes (focused border cyan, unfocused dark gray). Titles: the file name on the left; branch or `Status` on the right; `Rebase <range>`; `Details`. Verify with a TestBackend test at 120×30 on `ammend2` that the output contains `fix: release volume`, `Staged (6)` and `COMMIT_EDITMSG`, and not `Please enter`.
- [ ] 13.2 Draw the soft-wrapped editor for message, plain and raw text:
  - rows from `wrap::wrap`; selected text reversed; conflict-marker lines (a run of 6–7 `<`/`=`/`>`) white on red
  - `editor_top` adjusted so the cursor stays visible; `editor_width` stored
  - cursor column clamped to width−1, with a `ponytail:` comment
  - draw the cursor only when the left pane is focused and help is closed

  Verify a plain file with a 40-word line at 60 columns shows its last word.
- [ ] 13.3 Draw the rebase table:
  - Column widths: action 7, hash 8, subject the rest.
  - Rows: commit rows get an action color (pick green, reword blue, edit magenta, squash yellow, fixup cyan, drop red) and a `✎` prefix when a reword is pending; Other rows show the command word; Unknown rows are dimmed.
  - Wrapped subjects set row heights, and `table_top` keeps the selected row fully visible.
  - The inline editor text and cursor are drawn in the selected row.

  Verify `squash_fixture` at 140×30 shows `Rebase 36d7eda..aa619f8` and `54763e6`.
- [ ] 13.4 Draw the right pane:
  - Content: the status `render_lines`, or the details view (summary `N → M commits`, counts line, first-squash warning, hash, then `loading…` / `details unavailable` / message and badge rows).
  - Scrolling: clamp `right_scroll` to content minus height, store `right_page`, and show a `top-bottom/total` indicator in the bottom border when the content overflows.

  Verify: `merge_fixture2` scrolled to End shows `sync-ui` with `right_scroll < usize::MAX`, and the rebase fixture shows `14 → 14 commits`.
- [ ] 13.5 Verify narrow mode with TestBackend: `ammend2` at 80×30 shows `fix: release volume` but not `Staged`; after `ToggleFocus` it shows `Staged (6)` but not `fix: release volume`
- [ ] 13.6 Add `key_bar_entries(&App)` per state (help, inline, right focus, table, raw, message) with save/cancel/help first, `key_bar_line(entries, width)` dropping entries that don't fit, and the help overlay: global keys, per-layout keys, rebase command legend, reword prerequisite note, scrollable and centered over `Clear`. Verify:
  - entries `^S Save, Esc Cancel, ^H Help, Tab Cycle` at width 30 keep `Help` and drop `Cycle`
  - the rebase help contains `update-ref`
  - the message help contains `Ctrl+C`

## 14. Switch over (`src/main.rs`)

- [ ] 14.1 Rewrite `main.rs`:
  - Module list: context, details, keys, layout, message, rebase, reword, session, status, terminal, ui, wrap, writer.
  - Before the terminal is touched: install the panic hook, check the path exists (exit 1 if not), read the file, detect the context, call `App::new` with `git_dir` and `read_comment_char()`, and set `file_name`.
  - Event loop: draw, `event::poll(100ms)` with `poll_background` on timeout, `keys::map_event`, `apply`.
  - Save: serialize, drop the guard, `write_atomic`, `finish_save`, exit 0. Cancel: drop the guard, exit 1.

  Verify `cargo build` succeeds.
- [ ] 14.2 Delete `src/app.rs`, `src/renderer.rs` and `src/document.rs`; verify `cargo test` passes and `cargo clippy --all-targets` reports no warnings in new modules

## 15. Docs and release

- [ ] 15.1 Update `README.md`, keeping the user's uncommitted README edit (check `git diff README.md` first):
  - features (two-pane layout, status pane, rebase table with reorder/reword/raw)
  - keyboard shortcut tables per pane
  - recommend setting both `core.editor` and `sequence.editor`
  - note that mouse capture means Shift/Option-drag is needed for terminal text selection
  - remove the subject-counter and "no alternate screen" claims

  Verify by reviewing the rendered README.
- [ ] 15.2 Set `version = "1.0.0"` in `Cargo.toml`; verify `cargo build` and `target/debug/gitmedit --version` prints 1.0.0

## 16. End-to-end verification

- [ ] 16.1 Run `cargo test` and `cargo test -- --ignored`; verify all pass
- [ ] 16.2 In a scratch directory, copy each fixture to its git filename under a fake `.git/` (`COMMIT_EDITMSG`, `MERGE_MSG`, `SQUASH_MSG`, `rebase-merge/git-rebase-todo`) and open each with `target/debug/gitmedit` at ≥120 and 80 columns. Verify:
  - both panes appear, clicks and Alt+arrows move focus, and the wheel scrolls the pane under the pointer
  - End in `merge_fixture2` reaches `sync-ui`, and Ctrl+T works on narrow terminals
  - Esc leaves the file unchanged
  - Ctrl+S without edits writes a byte-identical file (`cmp` against the fixture)
- [ ] 16.3 In a temp repo with 3 commits, run `GIT_SEQUENCE_EDITOR=<abs>/target/debug/gitmedit GIT_EDITOR=<abs>/target/debug/gitmedit git rebase -i HEAD~2`. Verify:
  - reorder one row with Alt+Up, then reword the other with Enter, type a new subject and press Enter, then save with Ctrl+S
  - the message editor opens prefilled with the new subject and the original body
  - after Ctrl+S, `git log --format=%B -2` shows the new order, the new subject and the original body
  - `.git/gitmedit/reword` is empty
- [ ] 16.4 Verify the terminal is fully usable after save, after cancel, and after a panic (temporarily add `panic!()` to the Ctrl+T handler in a local debug build, then revert): echo works, the cursor is visible, prior output is back, no mouse escape codes appear on click, and the panic message is readable
- [ ] 16.5 Build with `cargo build --release` and time opening `ammend_fixture` then pressing Esc through `script`; verify wall time stays under 100 ms
- [ ] 16.6 Run `openspec validate two-pane-editor --strict`; verify it passes
