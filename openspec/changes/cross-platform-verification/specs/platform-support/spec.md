## Purpose

States which terminal environments the editor is expected to work in, and what correct behavior means
in each, so that platform-specific breakage is a specified failure rather than an unnoticed one.

## ADDED Requirements

### Requirement: Renders correctly on macOS terminals
Both layouts SHALL render without visual artifacts on the standard macOS terminals, including
Terminal.app and iTerm2.

#### Scenario: All layouts on macOS
- **WHEN** the message layout, the rebase layout, and standalone commit mode are opened on a macOS
  terminal
- **THEN** each renders with correct pane layout, colors, and cursor placement, and no stray escape
  sequences appear as visible text

#### Scenario: Resizing on macOS
- **WHEN** the terminal window is resized while the editor is open
- **THEN** the editor redraws to the new size without corrupting the display

### Requirement: Renders correctly on Windows Terminal
Both layouts SHALL render without visual artifacts on Windows Terminal.

#### Scenario: All layouts on Windows
- **WHEN** the message layout, the rebase layout, and standalone commit mode are opened on Windows
  Terminal
- **THEN** each renders with correct pane layout, colors, and cursor placement

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

### Requirement: Mouse and Alt keys work on every supported terminal
Click-to-focus, wheel scrolling, and the Alt-modified focus and reorder keys SHALL work on every
supported terminal, or the terminal-specific form they arrive in SHALL be recognized.

#### Scenario: Wheel over the status pane
- **WHEN** the user scrolls the wheel over the status pane on any supported terminal
- **THEN** the status pane scrolls

#### Scenario: Focus keys on macOS
- **WHEN** the user presses Alt+Right in Terminal.app or iTerm2 with default settings
- **THEN** focus moves to the right pane
