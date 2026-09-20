# Changelog

Versions follow [semantic versioning](https://semver.org). For this package, a breaking change is
one that changes how the editor reads or writes the files git hands it, or that removes a documented
key binding or command-line behavior.

This file is maintained by [knope](https://knope.tech) from conventional commit messages — new
sections are added when a release is prepared, not by hand.

## 1.1.0 (2026-09-20)

### Features

- rework readme, remove old gsd folders

## 1.0.0 (2026-09-15)

### Features

- Two-pane layout: message editor or rebase table on the left, git context on the right
- Message editor for commit, merge, squash, and tag messages, with the comment block parsed into the
  status pane and written back untouched
- Status pane showing branch and upstream state, conflicts, staged and unstaged files, submodules,
  untracked files, rebase progress, and `commit -v` diffs
- Interactive rebase table with letter keys, Tab cycling, Alt+↑/↓ reordering, inline reword, and a
  raw-text toggle
- Rebase details pane with the resulting history summary and the selected commit's message and files
- Soft-wrapped editing that never wraps lines in the file itself
- Mouse support for focusing and scrolling panes
- Running `gitmedit` with no arguments starts `git commit` with itself as the editor

### Fixes

- The terminal is restored exactly once, so panic messages stay readable
