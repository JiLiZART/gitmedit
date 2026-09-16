## ADDED Requirements

### Requirement: Reference is scoped to the current layout
The shortcut reference SHALL list the actions available in the current layout, grouped into global
keys, left-pane keys, and right-pane keys.

#### Scenario: Message layout
- **WHEN** the reference is opened in the message layout
- **THEN** it lists save, cancel, and help; focus switching (click, Alt+Left/Right, Esc) and the
  narrow-terminal toggle; text editing, clipboard, and undo keys; and status pane scrolling keys

#### Scenario: Rebase layout
- **WHEN** the reference is opened in the rebase layout
- **THEN** it lists the action letters and Tab cycle, reordering with Alt+Up/Down, inline reword with
  Enter, the raw text toggle Ctrl+E, and details pane scrolling
- **AND** it does not list text editing shortcuts that do not apply to the table

#### Scenario: Rebase command legend
- **WHEN** the reference is opened in the rebase layout
- **THEN** it includes git's rebase command legend (pick, reword, edit, squash, fixup, exec, break,
  drop, label, reset, merge, update-ref) with a one-line meaning for each

#### Scenario: Reword prerequisite stated
- **WHEN** the reference is opened in the rebase layout
- **THEN** it states that inline reword needs gitmedit as git's message editor as well

## MODIFIED Requirements

### Requirement: Shortcut reference on demand
The editor SHALL display a shortcut reference over the whole layout when the user presses Ctrl+H,
whichever pane has focus.

#### Scenario: Opening the reference
- **WHEN** the user presses Ctrl+H
- **THEN** a shortcut reference is displayed over both panes

### Requirement: Reference does not interfere with editing
While the shortcut reference is visible, the editor SHALL NOT apply keystrokes or mouse events to
either pane, so that reading the reference can never change what the user has written.

#### Scenario: Typing while the reference is open
- **WHEN** the user types ordinary characters while the reference is visible
- **THEN** the message and the rebase table are unchanged

#### Scenario: Scrolling the reference
- **WHEN** the reference is taller than the terminal and the user presses Down or scrolls the wheel
- **THEN** the reference scrolls and the panes beneath are unchanged

### Requirement: Reference is dismissible
The user SHALL be able to dismiss the shortcut reference and resume exactly where they left off, with
the same pane focused.

#### Scenario: Dismiss with Esc
- **WHEN** the user presses Esc while the reference is visible
- **THEN** the reference closes and editing resumes
- **AND** the editor does not exit

#### Scenario: Dismiss with the same shortcut
- **WHEN** the user presses Ctrl+H while the reference is visible
- **THEN** the reference closes and editing resumes

## REMOVED Requirements

### Requirement: Reference is scoped to the current operation
**Reason**: Protected lines no longer exist, so there are no restrictions to state; keys are now
grouped by layout and pane.
**Migration**: See "Reference is scoped to the current layout".
