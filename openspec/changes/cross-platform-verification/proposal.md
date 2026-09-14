# Cross-Platform Verification

> Carried over from GSD Phase 12 (`.planning/ROADMAP.md`) during the migration to OpenSpec. Not started.

## Why

The editor claims Linux and macOS support with Windows as a stretch goal, but that claim rests on the
cross-platform promises of its terminal library rather than on having been run anywhere. Terminal
behavior is exactly the area where those promises leak: escape sequence support, color handling, and
key event delivery all differ between terminal emulators, and the differences show up as visual
artifacts or duplicated keystrokes rather than as compile errors.

The Windows key-event question is the specific one worth naming. Windows consoles report key press
and key release as separate events, and a handler that acts on both fires every action twice — one
Tab cycling an action forward two steps, one Esc exiting instead of dismissing a help overlay. The
event loop already filters for press events, so the guard is present; what is missing is any evidence
it is sufficient on a real Windows console.

## What Changes

- Establish that both layouts render without artifacts on the target terminals.
- Establish that each keypress produces exactly one action on Windows.
- Establish that mouse clicks and wheel events reach the editor, and that the Alt-modified focus and
  reorder keys arrive in a form the editor recognizes, on each target terminal.
- Turn what is currently an assumption about platform support into a stated, checked requirement.

## Capabilities

### New Capabilities

- `platform-support`: the terminal environments the editor is expected to work in, and what "works"
  means in each.

### Modified Capabilities

None.

## Impact

- Likely no production code changes if the verification passes; fixes are scoped by what it finds.
- Requirements covered: PLAT-01, PLAT-02, PLAT-03.
- Depends on: every feature change before it. Verifying a moving target wastes the effort, so this
  runs once the other v1.1 work has settled.

**Note on scope:** this is the one change in the milestone whose outcome is knowledge rather than
behavior. It is specified as requirements anyway, because "renders correctly on Windows Terminal" is
a property users depend on and regressions in it are otherwise invisible.
