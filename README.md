# gitmedit

A fast, distraction-free git editor with nano-style shortcuts. Understands git context -- commits, merges, interactive rebase, and squash operations -- without bloat.

## Features

- Context-aware modes: automatically detects COMMIT_EDITMSG, MERGE_MSG, git-rebase-todo, and SQUASH_MSG
- Subject line intelligence: real-time character counter with color coding (green <=50, yellow 51-72, red >72)
- Interactive rebase table: structured display with action cycling (pick/squash/fixup/drop)
- Squash mode: read-only commit log with editable combined message
- Nano-style shortcuts: Ctrl+S save, Esc cancel, Ctrl+H help
- No alternate screen: prior terminal output remains visible (like nano)
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

```sh
# For commit messages
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

- **Commit:** `git commit` opens gitmedit with subject line counter and blank line detection
- **Rebase:** `git rebase -i HEAD~3` opens structured rebase table
- **Squash:** During rebase squash operations, shows commit log as read-only context

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+S | Save and exit |
| Esc | Cancel and exit |
| Ctrl+H | Toggle help overlay |
| Tab | Cycle rebase action (rebase mode) |

## Platform Support

- Linux, macOS, and Windows
- Uses crossterm for cross-platform terminal handling

## License

Licensed under MIT OR Apache-2.0
