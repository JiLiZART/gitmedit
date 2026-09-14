# Rebase View Word Wrapping

> Historical change. Reconstructed from `.planning/milestones/v1.0-phases/06-rebase-view-horizontal-scrolling/`
> during the migration from the GSD planning system to OpenSpec. Shipped 2026-04-02 as part of v1.0.

## Why

The rebase table gives the subject column whatever width is left after the action and commit columns.
On a narrow terminal, or with the long subjects conventional commit prefixes produce, that is not
enough room, and the subject is cut off. A truncated subject defeats the purpose of the table: the
user is choosing what to do with a commit and cannot see which commit it is.

The phase was scoped as horizontal scrolling and delivered word wrapping instead. Scrolling the table
sideways would have required tracking a horizontal offset, keeping it in sync with the selection, and
teaching the user a second navigation axis, all to read text that could simply be shown in full.

## What Changes

- Wrap long commit subjects in the rebase table across multiple terminal lines.
- Break at word boundaries where one exists within the available width, rather than mid-word.
- Give each row the height its wrapped subject needs, and account for variable row heights when
  scrolling the table.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `rebase-mode`: adds how long subjects are displayed within the table.

## Impact

- Subject wrapping and variable row heights in the rebase table renderer.
- No new dependencies, no change to the todo file format or to what is written back.
- Requirements covered: REBASE-02 (enhanced).
