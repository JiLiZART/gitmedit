# Commit Message Intelligence

> Historical change. Reconstructed from `.planning/milestones/v1.0-phases/03-commit-message-intelligence/`
> during the migration from the GSD planning system to OpenSpec. Shipped 2026-03-22 as part of v1.0.

## Why

Knowing git's context is what separates this editor from a general-purpose one. The first place to
spend that knowledge is the commit message itself: the widely followed convention of a short subject
line, a blank separator, and a body is easy to violate and invisible while typing in a plain editor.

The second is discoverability. Nano-style shortcuts are only obvious if the user is told what they
are, and the useful set differs by operation — the keys that matter while editing a message are not
the keys that matter while planning a rebase.

## What Changes

- Show a live character count for the subject line while the user types.
- Color that count by convention: comfortable up to 50 characters, cautionary to 72, over budget
  beyond that.
- Detect a missing blank line between the subject and the body and surface it to the user.
- Add a help overlay on Ctrl+H listing the shortcuts available in the current operation.
- Make the overlay modal: while it is visible it swallows input rather than leaking keystrokes into
  the message, and it is dismissed with Esc or Ctrl+H.

## Capabilities

### New Capabilities

- `commit-message`: feedback about the message being written, as distinct from the mechanics of
  editing text.
- `help-overlay`: the in-editor shortcut reference and its interaction with editing.

### Modified Capabilities

None.

## Impact

- Status-bar rendering gains subject-length feedback; a new overlay widget is added.
- No new dependencies.
- Requirements covered: COMMIT-01, COMMIT-02, COMMIT-03, COMMIT-06, HELP-01, HELP-02, HELP-03,
  HELP-04.
