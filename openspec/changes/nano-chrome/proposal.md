# Nano Chrome

> Carried over from GSD Phase 9 (`.planning/ROADMAP.md`) during the migration to OpenSpec. Not started.

## Why

Removing the subject counter left the status bar showing only a fixed string, and left the editor
with no indication of what file it has open or where. nano answers both questions permanently: a
title bar naming the file, and a command bar listing the keys. Adopting that shape finishes the
nano-likeness the editor claims and gives the removed status-bar space a purpose.

There is also a correctness reason to touch the shortcut reference now. It still tells the user that
conflict markers are read-only in merge mode, which stopped being true when conflict markers became
editable. The reference is where users go to learn what they can do, and it is currently wrong.

## What Changes

- A header bar across the top of every mode, showing the project folder on the left and the file
  being edited on the right.
- A command bar across the bottom listing the available shortcuts.
- Command bar contents that follow the current operation: rebase shows navigation and cycling,
  message modes show editing and clipboard keys.
- Correct the shortcut reference's claim about conflict markers in merge mode.

## Capabilities

### New Capabilities

- `editor-chrome`: the persistent header and command bars framing the editing area in every mode.

### Modified Capabilities

- `help-overlay`: corrects the read-only restrictions it reports for merge mode.

## Impact

- The renderer gains a three-region vertical layout — header, content, command bar — replacing the
  current two-region one. Every mode's content renderer must receive the middle region, and the
  scroll and cursor arithmetic that assumes the content area starts at the top of the frame has to
  be checked against the new offset.
- Requirements covered: UI-01, UI-02, UI-03, UI-04.
- Depends on: `restore-inline-rendering` is not a hard dependency, but drawing more chrome into the
  main screen buffer is easier to judge once the buffer question is settled.
