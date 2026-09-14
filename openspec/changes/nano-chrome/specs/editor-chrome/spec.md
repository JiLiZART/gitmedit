## Purpose

The persistent frame around the editing area — a header naming what is being edited and a command
bar naming the keys available — present in every mode the editor offers.

## ADDED Requirements

### Requirement: Header bar identifies the file
A header bar SHALL be displayed at the top of every mode, showing the project folder the file
belongs to and the name of the file being edited.

#### Scenario: Editing a commit message
- **WHEN** the editor opens a commit message inside a repository
- **THEN** the header shows the project folder name and the file name

#### Scenario: Every mode has the header
- **WHEN** the editor is in commit, merge, rebase, or squash mode
- **THEN** the header bar is present in all of them

#### Scenario: Narrow terminal
- **WHEN** the terminal is too narrow to show both the folder and the file name in full
- **THEN** the header degrades without wrapping or overflowing into the content area

### Requirement: Command bar lists available keys
A command bar SHALL be displayed at the bottom of every mode, listing the shortcuts available there.

#### Scenario: Shortcuts are visible without asking
- **WHEN** the editor is open in any mode
- **THEN** the command bar lists at least saving, cancelling, and opening the shortcut reference

### Requirement: Command bar follows the current operation
The command bar SHALL show the shortcuts that apply to the current git operation, and SHALL NOT
offer keys that do nothing there.

#### Scenario: Rebase mode
- **WHEN** the editor is in rebase mode
- **THEN** the command bar shows the navigation and action-cycling keys
- **AND** it does not offer free-text editing keys

#### Scenario: Message modes
- **WHEN** the editor is in commit, merge, or squash mode
- **THEN** the command bar shows the editing and clipboard keys

### Requirement: Chrome does not disturb the content area
Adding the header and command bar SHALL NOT change how content is scrolled or where the cursor
appears — the content area is simply the space that remains between them.

#### Scenario: Cursor position accounts for the header
- **WHEN** the user moves the cursor with the header bar present
- **THEN** the terminal cursor appears on the line the user is actually editing

#### Scenario: Scrolling accounts for the reduced height
- **WHEN** the content is taller than the space between the bars
- **THEN** scrolling keeps the cursor visible within that space, without hiding it behind either bar

#### Scenario: Rebase table height
- **WHEN** a rebase todo is displayed with the bars present
- **THEN** the table occupies the remaining space and its rows stay aligned with their content
