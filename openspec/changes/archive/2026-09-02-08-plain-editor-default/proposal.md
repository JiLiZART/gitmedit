# Plain Editor Default

> Historical change. Reconstructed from `.planning/phases/08-plain-editor-default/` during the
> migration from the GSD planning system to OpenSpec. Implemented 2026-05-14 as part of v1.1.

## Why

Protecting comment lines from editing was meant to keep users from breaking the structure git
expects. In practice it does the opposite of what a nano replacement should do. The user cannot
delete the instruction block git prepends, cannot move the cursor through it, and cannot use the
editor the way every other editor behaves. git strips those lines itself; there is nothing to
protect.

The commit message intelligence added earlier has the same problem from the other direction. A
character counter and a blank-line warning are opinions, and this editor's premise is that it stays
out of the way. Removing them takes the status bar back to what it should be — the available keys —
and leaves room for the chrome work that follows.

Merge messages need the middle position. The comment block git generates there is genuinely inert,
but conflict markers are the thing the user is resolving, and needing to delete them is normal.

## What Changes

- Introduce an explicit editing mode per git operation, decided when the file is opened.
- Commit and unknown files: every line is editable, including comments. Plain text editor behavior.
- Merge files: comments stay protected; conflict markers become editable, since resolving a conflict
  means removing them.
- Squash files: unchanged — both comments and the generated log stay protected.
- Remove the subject-line character counter, its color thresholds, and the blank-line warning.
- Retire the `commit-message` capability, which those three requirements were all of.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `text-editing`: line protection becomes conditional on the git operation instead of universal, and
  byte-for-byte preservation is scoped to the operations that still protect lines.
- `merge-mode`: conflict markers become editable.
- `commit-message`: all requirements removed; the capability is retired.

## Impact

- An editing mode is threaded from context detection into document parsing, changing which lines
  enter the editing surface and how the file is reassembled on save.
- Status bar rendering loses the counter and warning helpers.
- Requirements covered: EDIT-10, EDIT-11.

**Known inconsistency carried in, not introduced here:** the help overlay still tells the user that
conflict markers are read-only in merge mode, which this change made false. Fixing that text is left
to the chrome work in `nano-chrome`.
