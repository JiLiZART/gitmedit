# Rebase + Squash Modes

> Historical change. Reconstructed from `.planning/milestones/v1.0-phases/04-rebase-squash-modes/`
> during the migration from the GSD planning system to OpenSpec. Shipped 2026-03-23 as part of v1.0.

## Why

An interactive rebase todo is not prose. It is a small program — one instruction per line, each an
action followed by a commit — and editing it as free text is how people end up with a rebase that
aborts on a typo in `pick`. The editor already knows when a rebase todo is what it was handed, so it
can present that file as what it actually is: a list of commits with an action attached to each.

Squash messages have the opposite problem. The file git produces mixes the log of the commits being
combined with the message to be written. The log is context the user needs to see but must not edit,
because git generated it and will not read it back.

## What Changes

- Parse a rebase todo into typed lines: actions with their commit and subject, comments, and blank
  lines, preserving anything not recognized.
- Present the rebase todo as a structured table rather than as editable text, with one row per
  instruction.
- Cycle a row's action with a single key, through pick, squash, fixup, and drop.
- Navigate rows with the arrow keys, skipping comment lines, which are not selectable.
- Write the todo back in exactly the format git parses, preserving comment lines and line order.
- Detect a squash message file and split it into a read-only log header and an editable message.
- Present those as two panes, with the log clearly marked read-only.

## Capabilities

### New Capabilities

- `rebase-mode`: the structured interactive-rebase interface and the fidelity of the todo file it
  writes back.
- `squash-mode`: editing a squash message alongside the read-only log of the commits being combined.

### Modified Capabilities

None.

## Impact

- Rebase todo parsing and serialization added to the document layer, independent of the text
  document model.
- Table and dual-pane rendering added to the renderer.
- No new dependencies.
- Requirements covered: REBASE-01 through REBASE-06, SQUASH-01 through SQUASH-04.
