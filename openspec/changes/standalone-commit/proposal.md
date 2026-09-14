# Standalone Commit Mode

> Carried over from GSD Phase 11 (`.planning/ROADMAP.md`) during the migration to OpenSpec. Not started.

## Why

Every use of this editor so far begins with git launching it. But the common case — stage some
changes, write a message, commit — currently means typing `git commit`, waiting for git to write a
template file, and having git launch the editor on it. The editor already knows how to write a commit
message; it can be the entry point rather than the subprocess.

This also closes the loop on the tool's premise. If gitmedit is what you use to write commit
messages, running `gitmedit` in a repository should do exactly that.

## What Changes

- Running the editor with no arguments inside a git repository opens a commit message editor.
- Saving writes the message to a temporary file and invokes `git commit -F` with it.
- The editor exits with git's own exit code, so scripts and shells see what they would have seen from
  a direct `git commit`.
- Running with nothing staged reports that and exits before the editor opens, rather than after the
  user has written a message.
- When git rejects the commit — a hook failure, most often — git's output is shown to the user before
  the editor exits, rather than being swallowed.

## Capabilities

### New Capabilities

- `standalone-commit`: invoking the editor directly to author and submit a commit, rather than being
  invoked by git.

### Modified Capabilities

- `git-editor-contract`: the argument becomes optional, and its absence has meaning.

## Impact

- Argument parsing changes: no argument is a valid invocation, and its handling has to leave the
  existing single-path behavior untouched.
- The editor gains a subprocess call and a temporary file, in a process that until now only read and
  wrote one file it was handed.
- Requirements covered: COMMIT-10, COMMIT-11, COMMIT-12, COMMIT-13, COMMIT-14.
- Depends on: `restore-inline-rendering`. This change drops the terminal guard and then runs a
  subprocess that writes to the same terminal; doing that while the guard's cleanup can panic, and
  while the alternate screen may or may not be active, is how output ends up lost or garbled.

**Carried-over risk:** the failure paths here — nothing staged, empty message, hook rejection — are
the ones users will actually hit, and none of them can be covered by a unit test of the happy path.
They need an end-to-end check against a real repository.
