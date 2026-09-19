## Context

See proposal.md — Why. `src/main.rs` parses one required `path` with clap and runs the editor on it.
When git runs the editor for a commit it passes `.git/COMMIT_EDITMSG`, and gitmedit already opens that
in the message layout with the status pane. `GIT_EDITOR` takes precedence over `core.editor` and
`$EDITOR`. git runs the editor command through the shell, so the value is a shell word.

## Goals / Non-Goals

**Goals:**
- `gitmedit` with no arguments behaves exactly like `git commit` with gitmedit as the editor.

**Non-Goals:**
- Passing git options through (for example `gitmedit --amend`). They would collide with gitmedit's own
  flags; a separate decision.
- Pre-checks for "inside a repository" or "anything staged". git already reports both, with the right
  message and exit code.
- Running `git add` or any staging.

## Decisions

**Delegate the whole flow to `git commit` with `GIT_EDITOR` pointing at ourselves.** The no-argument
path spawns `git commit` with `GIT_EDITOR` set to the shell-quoted path of the running binary, from
`std::env::current_exe()`. It inherits stdin, stdout, and stderr, waits, and exits with git's code; a
child killed by a signal exits 1. git then launches gitmedit on `COMMIT_EDITMSG` like any other commit.

Alternative, from the removed `standalone-commit` change: open an empty editor, then run
`git commit -F <tempfile>`. Rejected on four counts:
- It loses git's template and status block, so the status pane would be empty.
- It re-implements git's nothing-staged and repository checks.
- It needs a temp file.
- It needs careful terminal teardown before git writes its output.

**The parent never touches the terminal.** No `TerminalGuard` is created on this path, so git and its
hooks write to a normal terminal. The only terminal setup happens inside the child gitmedit, which
already restores on every exit path.

**Quote with single quotes.** The path is wrapped in `'…'`, and each embedded `'` becomes `'\''`. That
is valid for the POSIX `sh` git uses on Linux and macOS. On Windows, git for Windows also runs editors
through its bundled `sh`.

**Spawn failure exits 127.** When git cannot be executed, gitmedit prints
`gitmedit: could not run git: <error>` to stderr and exits 127, the shell convention for a command that
cannot be found.

**The argument becomes `Option<PathBuf>`.** clap keeps rejecting extra arguments with a usage error and
exit code 2, so "more than one argument" needs no custom code.

## Risks / Trade-offs

- [`current_exe()` can point at a deleted or replaced binary during `cargo install` upgrades] → Rare
  and self-correcting on the next run. git reports the editor failure.
- [A user's `GIT_EDITOR` is overridden for this invocation] → Intended: the command's promise is
  "commit in gitmedit". The environment of the parent shell is not changed.
- [Nested invocation: gitmedit started by git with no path] → Cannot happen; git always passes the
  file path.
