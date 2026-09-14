# Restore Inline Rendering

> Supersedes GSD Phase 7 (`.planning/phases/07-foundations-fix/`), which was implemented, marked
> complete on 2026-04-22, and then reverted by commit `8dd89f1` without the tracking being updated.
> Verified against the tree during the OpenSpec migration: both defects are present today.

## Why

The editor is supposed to draw into the main screen buffer, the way nano does, so that the `git
commit` output the user was just reading stays visible above it and remains in scrollback after it
exits. Today it enters the alternate screen instead, which wipes that context on entry and restores a
screen with no trace of the edit on exit. The README still advertises the intended behavior.

The terminal guard has a second defect from the same revert: its cleanup path calls `.unwrap()`. A
`Drop` implementation that panics while another panic is unwinding aborts the process, and the
cleanup that aborts is the cleanup responsible for giving the user their terminal back. The comment
directly above the call says errors must be suppressed because `Drop` must not panic; the code does
the opposite.

Both need to be fixed before standalone commit mode, which drops the guard and then hands control to
a `git commit` subprocess — a sequence that assumes teardown is reliable.

## What Changes

- Stop entering the alternate screen; construct the terminal backend without it.
- Stop enabling mouse capture, which nothing in the event loop consumes.
- Make the guard's cleanup path suppress errors instead of unwrapping them.
- Make the panic path leave the terminal fully usable, not merely out of raw mode.
- Make the regression detectable: the existing PTY test asserting no alternate-screen escape
  sequences is `#[ignore]`d, which is why the revert passed a full verification with 96 green tests.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `git-editor-contract`: adds inline rendering and strengthens what "terminal is restored" means.

## Impact

- `src/terminal.rs`: guard construction and `Drop`, and the panic hook.
- `tests/terminal_integration.rs`: the ignored PTY test needs to become something that actually runs
  in the ordinary suite, or the guarantee stays unguarded.
- Requirements covered: IO-06, IO-07, and the full form of IO-03.
- Unblocks: `standalone-commit`.

**Root cause not yet established.** Whether `8dd89f1` was a deliberate reversal — the alternate
screen was wanted, or the Phase 7 fix broke something in practice — or an accident during the
worktree merge that preceded it has not been determined. If it was deliberate, this change is wrong
and the specs and README should change instead. Settle that before implementing.
