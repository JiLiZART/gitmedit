# help-overlay Specification

## Purpose
Makes the editor's shortcuts discoverable from inside the editor, showing the set that applies to
the git operation currently in progress.

## Requirements

### Requirement: Shortcut reference on demand
The editor SHALL display a shortcut reference when the user presses Ctrl+H.

#### Scenario: Opening the reference
- **WHEN** the user presses Ctrl+H
- **THEN** a shortcut reference is displayed over the current view

### Requirement: Reference is scoped to the current operation
The shortcut reference SHALL list only the actions available in the current git operation, and SHALL
state the editing restrictions that apply there.

#### Scenario: Editing a message
- **WHEN** the reference is opened while editing a commit or merge message
- **THEN** it lists the text editing, clipboard, save, and cancel shortcuts

#### Scenario: Planning a rebase
- **WHEN** the reference is opened while editing a rebase todo
- **THEN** it lists the rebase navigation and action-cycling shortcuts
- **AND** it does not list free-text editing shortcuts, which do not apply there

#### Scenario: Restrictions are stated
- **WHEN** the current operation protects some lines from editing
- **THEN** the reference states which lines are read-only

### Requirement: Reference does not interfere with editing
While the shortcut reference is visible, the editor SHALL NOT apply keystrokes to the message, so
that reading the reference can never corrupt what the user has written.

#### Scenario: Typing while the reference is open
- **WHEN** the user types ordinary characters while the reference is visible
- **THEN** the message is left unchanged

### Requirement: Reference is dismissible
The user SHALL be able to dismiss the shortcut reference and resume editing exactly where they left
off.

#### Scenario: Dismiss with Esc
- **WHEN** the user presses Esc while the reference is visible
- **THEN** the reference closes and editing resumes
- **AND** the editor does not exit

#### Scenario: Dismiss with the same shortcut
- **WHEN** the user presses Ctrl+H while the reference is visible
- **THEN** the reference closes and editing resumes
