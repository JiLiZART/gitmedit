## Context

gitmedit is invoked by git as a subprocess with one argument and is judged on two outputs: the bytes
in the file, and the exit code. See proposal.md — Why. The design questions at this stage are all
about failure modes: what happens to the user's terminal and to their file when something goes wrong
mid-edit.

## Goals / Non-Goals

**Goals:**

- A terminal that is always usable after the process exits, on every path.
- A file that is either fully old or fully new, never half-written.
- Startup latency low enough that the editor is not noticed as an extra step.

**Non-Goals:**

- Editing features beyond loading and saving — text editing arrives in a later change.
- Any git interaction beyond reading and writing the file git handed over. No git library is linked.

## Decisions

**Single crate, no workspace.** One binary with no library split. Keeps compile times and cognitive
overhead low for a tool this size. Alternative considered: a `crates/gitmedit` workspace layout,
rejected as premature structure for a single binary.

**No alternate screen.** The editor draws into the main screen buffer, the way nano does, so the
user's prior terminal output stays visible above the editor after it exits. Alternative considered:
the conventional TUI approach of entering the alternate screen, rejected because it erases the
context — the `git commit` output the user was just looking at — and makes the tool feel heavier
than the nano it replaces. This also rules out `ratatui::init()`, which enters the alternate screen
as part of its setup; the backend is constructed manually instead.

**RAII guard for terminal state.** Raw mode is entered by constructing a guard value and left by
dropping it, rather than by paired setup/teardown calls. Rust runs `Drop` during unwinding, so the
restore happens even on an unexpected panic. Alternative considered: explicit teardown at each exit
point, rejected because every future `return` or `?` becomes a chance to leak raw mode.

**Panic hook installed before any terminal mutation.** The guard alone is not enough: a panic prints
its message through the hook, and if raw mode is still on the message renders as a staircase. The
hook disables raw mode first, then delegates to the original hook so the message and backtrace are
preserved.

**Atomic write via temp file plus rename.** Content is written to a temporary file next to the
target and then renamed over it. `rename` is atomic within a filesystem, so a crash leaves either the
original or the complete new content. Alternative considered: truncate-and-write in place, rejected
because a crash mid-write destroys the user's commit message with no recovery.

**Terminal restored before the file write.** On save the guard is dropped before writing, so that if
the write fails the error message is printed to a terminal already back in cooked mode.

## Risks / Trade-offs

- Drawing into the main buffer means the editor's frames scroll into the user's scrollback → accepted
  deliberately; this is the nano-like behavior being sought, not a defect.
- Constructing the crossterm backend manually instead of using the ratatui helper means future
  ratatui upgrades may change setup requirements under us → mitigated by keeping all terminal setup
  in a single small module.
- The no-alternate-screen decision is invisible to unit tests; nothing in the ordinary test suite
  fails if a later change reintroduces the alternate screen → mitigated by a PTY-based integration
  test that inspects the emitted escape sequences. That test requires a real PTY and is therefore
  `#[ignore]`d, which weakens the guard considerably.

## Migration Plan

Not applicable — initial change, no prior behavior to migrate.
