## Purpose

States which terminal environments the editor is expected to work in, and what correct behavior means
in each, so that platform-specific breakage is a specified failure rather than an unnoticed one.

## ADDED Requirements

### Requirement: Renders correctly on macOS terminals
Every mode SHALL render without visual artifacts on the standard macOS terminals, including
Terminal.app and iTerm2.

#### Scenario: All modes on macOS
- **WHEN** commit, merge, rebase, squash, and standalone commit modes are opened on a macOS terminal
- **THEN** each renders with correct layout, colors, and cursor placement, and no stray escape
  sequences appear as visible text

#### Scenario: Resizing on macOS
- **WHEN** the terminal window is resized while the editor is open
- **THEN** the editor redraws to the new size without corrupting the display

### Requirement: Renders correctly on Windows Terminal
Every mode SHALL render without visual artifacts on Windows Terminal.

#### Scenario: All modes on Windows
- **WHEN** commit, merge, rebase, squash, and standalone commit modes are opened on Windows Terminal
- **THEN** each renders with correct layout, colors, and cursor placement

### Requirement: One keypress produces one action
Each keypress SHALL produce exactly one action on every supported platform, including consoles that
report key press and key release as separate events.

#### Scenario: Cycling a rebase action on Windows
- **WHEN** the user presses Tab once on a selected rebase instruction on Windows
- **THEN** the action advances exactly one step in the cycle

#### Scenario: Dismissing the shortcut reference on Windows
- **WHEN** the user presses Esc once while the shortcut reference is visible on Windows
- **THEN** the reference is dismissed and the editor remains open

#### Scenario: Typing on Windows
- **WHEN** the user types a character once
- **THEN** exactly one character is inserted
