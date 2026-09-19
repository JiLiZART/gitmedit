## Why

The editor shows git's comment block as raw, uniform text inside the message the user is writing, so
the information they opened the editor to check — staged files, conflicts, branch state, rebase
progress — is buried in boilerplate and mixed into editable text. The rebase table can change actions
but not order commits, reword them, or show what a commit contains. Rebuilding the editor as two panes
puts what the user writes on the left and what git is telling them on the right, readable and
scrollable however many files are involved.

## What Changes

- **BREAKING** Full-screen two-pane layout on the alternate screen with mouse capture, replacing the
  intended inline (nano-style, no alternate screen) rendering. Left pane edits; right pane shows
  context. Focus moves by click, Alt+Left/Right (or `ESC b`/`ESC f`), and Esc; the wheel scrolls the
  pane under the pointer. Below 100 columns only one pane is shown and Ctrl+T swaps. A key bar lists
  the keys for the focused pane.
- **BREAKING** Message files (commit, merge, squash, tag) are split on open: the editor holds only
  the message; the comment block is kept aside, shown parsed in the right pane, and written back
  verbatim after the message on save. Comment lines are no longer editable or protected in the
  editor, and the merge and squash special modes are removed.
- New status pane: branch summary, merge and rebase state, squash notes, conflicts, staged, unstaged,
  submodules, untracked, and verbose diff as sections with counts and colored M/A/D/R/C/T/U badges.
- Soft wrap in the message editor replaces horizontal scrolling; the cursor moves by display row.
- Rebase table gains letter keys for actions, Alt+Up/Down reordering, inline reword (Enter), and a
  raw-text toggle (Ctrl+E). Reworded messages are stored under the git directory and prefilled when
  git stops to reword that commit.
- Rebase right pane shows a result summary and the selected commit's message and changed files,
  loaded in the background.
- Terminal restoration covers mouse capture, alternate screen, cursor, and raw mode on every exit
  path, and never panics.
- Help overlay lists keys per layout and includes git's rebase command legend.

## Capabilities

### New Capabilities

- `pane-layout`: two-pane full-screen layout, focus, mouse and keyboard scrolling, narrow terminals,
  and the key bar.
- `status-pane`: parsing git's comment block into sections and presenting it as a scrollable,
  colored status view.

### Modified Capabilities

- `text-editing`: message/trailer split and reassembly replace line classification and protection;
  soft wrap replaces horizontal scrolling; conflict markers are highlighted but plain text.
- `rebase-mode`: letter actions, reordering, inline reword with stored messages, raw toggle, and the
  details pane; comment lines leave the table.
- `git-editor-contract`: full terminal restoration including alternate screen and mouse capture;
  startup must not wait on commit details.
- `git-context-detection`: tag detected as its own operation, layout selection, git directory
  derivation, and detection of a reword in progress.
- `help-overlay`: per-layout key reference with rebase command legend; restriction notes removed.
- `distribution`: documentation recommends configuring gitmedit as both editors for inline reword.
- `merge-mode`: removed; merge messages use the message layout.
- `squash-mode`: removed; squash messages use the message layout.

## Impact

- Code: `src/app.rs`, `src/renderer.rs`, and `src/document.rs` are replaced by new modules for
  wrapping, message splitting, status parsing, rebase todo handling, reword storage, commit details,
  layout, session state, key mapping, and rendering; `src/main.rs`, `src/terminal.rs`, and
  `src/context.rs` change.
- Tests: the PTY integration test flips to assert alternate screen enter/leave.
- Files written: new `<gitdir>/gitmedit/reword/<commit>` files during interactive rebases.
- Processes: `git log`/`git show` subprocess calls for reword bodies and commit details.
- Dependencies: none added.
- Other changes: `standalone-commit` and `cross-platform-verification` already reference this layout.
- `openspec/config.yaml` states the two-pane constraint.
- `README.md` needs its features and shortcuts updated.
