## Context

The behavior being restored was designed and implemented once already; see the design of the initial
change, `2026-09-02-01-git-contract-tui-shell`. See proposal.md — Why for what regressed. The
interesting design question here is not how to render inline — that is a handful of lines — but why
the regression survived a full verification pass, and what to change so the next one does not.

## Goals / Non-Goals

**Goals:**

- Restore inline rendering and panic-safe teardown.
- Leave behind a test that fails if either regresses, running in the ordinary suite.

**Non-Goals:**

- Any change to how content is drawn once the backend exists. Layout and widgets are untouched.
- Deciding the mouse-capture question beyond removing it: nothing reads mouse events today, so it is
  removed as dead setup rather than reworked.

## Decisions

**Suppress errors in cleanup, do not propagate them.** `Drop` cannot return a result, and a panic
inside `Drop` during an unwind aborts the process. Cleanup discards its errors. The trade-off is a
terminal restore that can fail silently; that is strictly better than an abort that skips the rest of
cleanup, and there is nowhere useful to report the error to at that point anyway.

**The panic hook restores whatever the guard switched on.** The hook and the guard must agree about
what state the terminal is in. The previous implementation encoded "we never enter the alternate
screen" as a comment in the hook and a `LeaveAlternateScreen` call that was deliberately omitted,
which is exactly what made the revert invisible — the code that entered the alternate screen and the
code responsible for leaving it were in different functions with no shared source of truth.

**The regression test has to run without a PTY.** The existing PTY test is `#[ignore]`d because it
needs a real pseudo-terminal, so it did not run during the verification that passed with the revert
in place. An assertion that never executes is not coverage. Options, in order of preference:

1. Assert on the escape sequences the terminal setup emits into an in-memory buffer, which needs no
   PTY and runs everywhere.
2. Keep the PTY test but un-ignore it, and skip it at runtime only when PTY allocation actually
   fails, so it runs by default wherever it can.

Option 1 is preferred because it makes the guarantee a unit-level property of the setup code.

## Risks / Trade-offs

- The revert may have been deliberate → this is the real risk, and it is not a technical one. If
  someone reverted because the alternate screen was actually wanted, implementing this change
  reverts their reversal. Mitigation: establish intent before starting; see proposal.md.
- Suppressed cleanup errors could hide a genuine terminal problem → accepted; the alternative aborts
  the process during cleanup, which is worse in every case.

## Migration Plan

No data migration. On upgrade the editor stops clearing the screen when it opens; users who became
used to the alternate-screen behavior will see the change immediately.

## Open Questions

- Whether the PTY test should be kept at all once an in-memory assertion exists, or kept as a
  slower end-to-end check. Deferred: it changes neither the specs nor the approach.
