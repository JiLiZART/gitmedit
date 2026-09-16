# gitmedit

A fast, distraction-free git editor with nano-style shortcuts. It understands git context — commits, merges, squashes, tags, and interactive rebase — and shows what git is telling you next to what you are writing.

## Features

- Two-pane layout: edit on the left, context on the right
- Message editor for commit, merge, squash, and tag messages, with the `#` comment block kept out of your way and written back untouched on save
- Status pane: branch and upstream state, conflicts, staged and unstaged files, submodules, untracked files, rebase progress, and `commit -v` diffs, as colored sections with file counts and M/A/D/R/U badges, scrollable to the last file
- Soft-wrapped editing: long lines wrap on screen, never in the file
- Interactive rebase table:
  - letter keys and Tab set actions
  - Alt+↑/↓ reorders commits
  - Enter rewords a subject inline
  - Ctrl+E switches to raw text
- Rebase details pane: the resulting history summary and the selected commit's message and changed files
- Mouse support: click a pane to focus it, scroll the pane under the pointer
- Fast startup: under 100ms

## Installation

### From crates.io

```sh
cargo install gitmedit
```

### From source

```sh
git clone https://github.com/jilizart/gitmedit.git
cd gitmedit
cargo install --path .
```

## Configuration

### Set as default git editor

Set both. Inline reword in the rebase table needs gitmedit as the message editor too, because git
asks the message editor for the new message when it reaches the commit.

```sh
# For commit, merge, squash, and tag messages
git config --global core.editor gitmedit

# For interactive rebase
git config --global sequence.editor gitmedit
```

### Verify configuration

```sh
git config --global --get core.editor    # should print: gitmedit
git config --global --get sequence.editor # should print: gitmedit
```

## Usage

- **Commit without arguments:** run `gitmedit` in a repository to start `git commit` with gitmedit as the editor for that run only, whatever `core.editor` says. git writes the usual template, so the message opens with the status pane; hooks, `commit.template` and signing all apply, and gitmedit exits with git's own exit code and output ("nothing to commit", a rejected hook, and so on).
- **Commit, merge, squash, tag:** the message is on the left and git's status on the right. Below 100 columns, only one pane is shown; press Ctrl+T to switch.
- **Rebase:** `git rebase -i HEAD~5` opens the rebase table. Set actions, reorder, or reword, then save with Ctrl+S.
  - When git reaches a reworded commit, gitmedit opens with the new subject already filled in; confirm with Ctrl+S.

gitmedit takes over the mouse while it runs. To select terminal text with the mouse, hold Shift while dragging (Option in iTerm2).

## Keyboard Shortcuts

### Everywhere

| Shortcut | Action |
|----------|--------|
| Ctrl+S | Save and exit |
| Esc | Cancel and exit (left pane) / back to the left pane (right pane) |
| Ctrl+H | Toggle help |
| Alt+← / Alt+→ | Focus left / right pane |
| Ctrl+T | Switch pane |

### Message editor

| Shortcut | Action |
|----------|--------|
| Ctrl+C / Ctrl+X / Ctrl+V | Copy / cut / paste |
| Ctrl+U | Delete line |
| Ctrl+W / Ctrl+D | Delete previous / next word |
| Ctrl+Z / Ctrl+Y | Undo / redo |

### Rebase table

| Shortcut | Action |
|----------|--------|
| ↑ / ↓, Home / End | Select instruction |
| p r e s f d | Pick / reword / edit / squash / fixup / drop |
| Tab | Cycle pick → squash → fixup → drop |
| Alt+↑ / Alt+↓ | Move instruction |
| Enter | Reword subject inline (Enter confirms, Esc discards) |
| Ctrl+E | Toggle raw text editing |

### Status and details pane

| Shortcut | Action |
|----------|--------|
| ↑ / ↓ | Scroll one line |
| PgUp / PgDn | Scroll one page |
| Home / End | Jump to top / bottom |

## Platform Support

- Linux and macOS, with Windows as a stretch goal
- Uses crossterm for cross-platform terminal handling

## License

Licensed under MIT OR Apache-2.0
