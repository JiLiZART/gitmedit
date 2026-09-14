# Git Editor Contract + TUI Shell

> Historical change. Reconstructed from `.planning/milestones/v1.0-phases/01-git-contract-tui-shell/`
> during the migration from the GSD planning system to OpenSpec. Shipped 2026-03-18 as part of v1.0.

## Why

git invokes `core.editor` with a single file path and judges the result by two things: the bytes
left in that file, and the process exit code. Every richer behavior gitmedit wants to offer sits on
top of that contract, so the contract has to be established and correct before anything else exists.

The editor also has to survive its own failures. A TUI that panics while the terminal is in raw mode
leaves the user with a shell that no longer echoes input — an unacceptable outcome for a tool wired
into `git commit`.

## What Changes

- Read the file to edit from the first command-line argument; fail loudly if it does not exist.
- Write edited content back to the same path atomically (write to a temp file, then rename), so a
  crash mid-write cannot truncate a commit message or a rebase todo.
- Exit 0 on save, exit 1 on cancel or error, so git can distinguish "use this message" from "abort".
- Wrap raw-mode entry in an RAII guard and install a panic hook, so the terminal is restored on both
  the normal path and the panic path.
- Detect which git operation is in progress from the filename and expose it as a typed context.
- Keep startup fast enough that the editor does not feel heavier than nano.

## Capabilities

### New Capabilities

- `git-editor-contract`: the process-level contract with git — argument handling, atomic write-back,
  exit codes, terminal restoration, and startup latency.
- `git-context-detection`: recognizing which git operation invoked the editor, from the filename.

### Modified Capabilities

None — this is the first change.

## Impact

- New crate `gitmedit` with `main.rs`, `terminal.rs`, `writer.rs`, `context.rs`.
- Dependencies introduced: ratatui, crossterm, clap, anyhow.
- Requirements covered: IO-01, IO-02, IO-03, IO-04, IO-05, CTX-01, CTX-02, PERF-01.
