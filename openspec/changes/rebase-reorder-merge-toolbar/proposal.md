# Rebase Reordering + Merge Toolbar

> Carried over from GSD Phase 10 (`.planning/ROADMAP.md`) during the migration to OpenSpec. Not started.

## Why

The rebase table lets the user change what happens to each commit but not the order they happen in,
which is half of what interactive rebase is for. Reordering was deferred at v1.0 as too complex next
to action cycling; with the table stable it is the obvious next capability. Exec instructions have
the same gap from the other side: they are visible, they are inert, and their command — the only
part worth editing — cannot be changed.

Merge mode wastes the information git already put in the file. The generated comment block lists the
files that conflicted, and the user has to read past the instruction text to find it. With a command
bar in place there is somewhere to put that list.

## What Changes

- Move the selected rebase instruction up or down with Alt+Up and Alt+Down.
- Keep the selection on the instruction that moved, not on the position it left.
- Rebuild the selectable-row index after every move, so navigation cannot desynchronize from the
  instruction list.
- Preserve comment lines and their positions across reordering, and never lose or duplicate an
  instruction.
- Edit the command of an exec instruction inline: Enter opens it, Ctrl+S commits the edit, Esc
  discards it.
- Parse the conflicted-file list out of the merge message's comment block and show it in merge mode.
- Show nothing at all when there is no conflict block, rather than an empty or broken area.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `rebase-mode`: adds instruction reordering and exec command editing.
- `merge-mode`: adds display of the conflicted files git recorded in the message.

## Impact

- Reordering mutates the instruction list, which every other rebase behavior reads; the selectable
  index and the selection both have to follow it.
- Inline exec editing introduces a second input state to rebase mode, which until now consumed only
  navigation keys.
- Requirements covered: REBASE-10, REBASE-11, REBASE-12, REBASE-13, UI-05, UI-06.
- Depends on: `nano-chrome`, which provides the area the conflicted-file list is displayed in.

**Carried-over risk:** write the test for `pick / comment / pick` with the second instruction moved
up *before* implementing the move. Moving an instruction past a comment is where an implementation
that swaps adjacent list positions and an implementation that swaps adjacent *instructions* disagree,
and only one of them preserves the file.
