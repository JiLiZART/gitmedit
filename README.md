<div align="center">

# gitmedit

**Your git editor should show you what git is telling you.**

A fast, distraction-free terminal editor for commit messages and interactive rebase —
with nano-style shortcuts, so there is nothing to learn.

[![CI](https://github.com/JiLiZART/gitmedit/actions/workflows/ci.yaml/badge.svg)](https://github.com/JiLiZART/gitmedit/actions/workflows/ci.yaml)
[![crates.io](https://img.shields.io/crates/v/gitmedit.svg)](https://crates.io/crates/gitmedit)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

</div>

## The problem

You type `git commit`. vim opens. Somewhere below your cursor, git has written out everything you
need to know — which files are staged, which are not, whether you are mid-rebase — as a wall of `#`
comments you have to scroll past, and that you must not accidentally edit.

Then you run `git rebase -i` and get a text file where one typo in the word `squash` breaks the
whole operation.

## The fix

gitmedit puts git's context in its own pane, next to what you are writing.

```
┌──────────────────────────────┬──────────────────────────────┐
│ Add retry to the fetch path  │ On branch feature/retry      │
│                              │ Your branch is ahead by 2    │
│ The API times out under load │                              │
│ so we back off and retry     │ Staged (2)                   │
│ three times.                 │   M  src/fetch.rs            │
│                              │   A  src/backoff.rs          │
│                              │                              │
│                              │ Not staged (1)               │
│                              │   M  README.md               │
├──────────────────────────────┴──────────────────────────────┤
│ ^S Save  ^H Help  ^T Switch pane                            │
└─────────────────────────────────────────────────────────────┘
```

The comment block never appears in your editor — and is written back to git untouched.

## Why you'll like it

- **Nothing to learn.** Ctrl+S saves, Esc cancels, Ctrl+Z undoes. The shortcuts you already use.
- **Context where you can read it.** Branch and upstream state, conflicts, staged and unstaged
  files, submodules, untracked files, rebase progress, and `commit -v` diffs — colored, counted,
  badged M/A/D/R/U, and scrollable.
- **Interactive rebase that can't be fat-fingered.** Press `s` to squash, `d` to drop, Tab to cycle.
  Alt+↑/↓ reorders commits. Enter rewords a subject right there in the table.
- **It never mangles your file.** Roundtrip fidelity beats every other concern: comment lines,
  rebase todos, and formatting come back exactly as git wrote them.
- **Long lines wrap on screen, never in the file.** Your 72-column discipline stays yours to choose.
- **Mouse works.** Click a pane to focus it, scroll the one under your pointer.
- **Starts in under 100ms.** It competes with nano on launch feel, not on features you'll never use.

## Quick start

```sh
cargo install gitmedit

git config --global core.editor gitmedit      # commit, merge, squash, tag messages
git config --global sequence.editor gitmedit  # interactive rebase

gitmedit                                       # commit right now, no flags needed
```

That's it. Next `git commit` or `git rebase -i` opens in gitmedit.

### Install from source

```sh
git clone https://github.com/jilizart/gitmedit.git
cd gitmedit
cargo install --path .
```

### Verify your configuration

```sh
git config --global --get core.editor     # should print: gitmedit
git config --global --get sequence.editor # should print: gitmedit
```

Set **both**. Inline reword in the rebase table needs gitmedit as the message editor too, because
git asks the message editor for the new message when it reaches the commit.

## Usage

- **Commit without arguments** — run `gitmedit` in a repository and it starts `git commit` with
  itself as the editor for that run only, whatever `core.editor` says. git writes the usual
  template, so the message opens with the status pane; hooks, `commit.template` and signing all
  apply, and gitmedit exits with git's own exit code and output ("nothing to commit", a rejected
  hook, and so on).
- **Commit, merge, squash, tag** — message on the left, git's status on the right. Below 100
  columns only one pane shows; press Ctrl+T to switch.
- **Rebase** — `git rebase -i HEAD~5` opens the rebase table. Set actions, reorder, or reword, then
  save with Ctrl+S. When git reaches a reworded commit, gitmedit opens with the new subject already
  filled in; confirm with Ctrl+S.

gitmedit takes over the mouse while it runs. To select terminal text, hold Shift while dragging
(Option in iTerm2).

## Keyboard shortcuts

Press **Ctrl+H** any time for the same list in-app.

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

## Platform support

- Linux and macOS, with Windows as a stretch goal
- Built on [crossterm](https://github.com/crossterm-rs/crossterm) for cross-platform terminal handling
- The terminal is always restored on exit — including on panic

## Contributing

Pull requests welcome. Two things to know:

- Commit messages follow [conventional commits](https://www.conventionalcommits.org) — they decide
  the next version number.
- CI runs `cargo fmt --check`, `cargo clippy -- -D warnings`, and the test suite on Linux and macOS.

## Releases

Releases are automated with [knope](https://knope.tech). Nothing is published by hand.

- **Commit messages decide the version.** `feat:` raises the minor version, `fix:` the patch, and
  `!` or a `BREAKING CHANGE:` footer the major. A `chore:` or `docs:` commit releases nothing.
- **Every push to `main` refreshes a release pull request** titled `chore: prepare release <version>`.
  It shows the next version, the updated `Cargo.toml`/`Cargo.lock`, and the `CHANGELOG.md` entry that
  would be published.
- **Merging that pull request is the release.** It publishes the crate to crates.io, tags the commit,
  and creates a GitHub Release with the changelog entry as its notes.
- **The `release` branch belongs to the automation** and is force-pushed on every run. Never branch
  from it or commit to it.
- Edit `CHANGELOG.md` only for older entries; new sections are written by the release tool.

### Required repository secrets

| Secret | Used by | Scope |
|--------|---------|-------|
| `RELEASE_PAT` | `prepare_release.yaml` | Fine-grained token for this repository with **contents: write** and **pull requests: write**. A pull request opened with the default `GITHUB_TOKEN` would not trigger the release workflow when it merges, which is why this one exists. |
| `CARGO_REGISTRY_TOKEN` | `release.yaml` | A crates.io API token with publish rights for `gitmedit`. |

`release.yaml` uses the default `GITHUB_TOKEN` for the tag and release; no extra secret is needed
there.

## License

Licensed under MIT OR Apache-2.0.
