# rebase-mode Specification

## Purpose
Presents an interactive rebase todo as the list of instructions it is, letting the user change what
happens to each commit without hand-editing a file git will refuse to parse if it is malformed.

## Requirements

### Requirement: Todo file is parsed into typed lines
The editor SHALL parse a rebase todo into typed lines, distinguishing instruction lines — an action
followed by a commit reference and a subject — from comment lines and blank lines.

#### Scenario: Instruction line
- **WHEN** a line reads as an action followed by a commit reference and a subject
- **THEN** it is parsed as an instruction with those three parts recorded separately

#### Scenario: Abbreviated actions
- **WHEN** an instruction uses git's single-letter action abbreviation
- **THEN** it is parsed as the same action as the long form

#### Scenario: Comment line
- **WHEN** a line begins with the configured comment character
- **THEN** it is preserved as a comment and not treated as an instruction

#### Scenario: Unrecognized line
- **WHEN** a line matches neither form
- **THEN** it is preserved unchanged rather than discarded or reinterpreted

#### Scenario: Subject containing spaces
- **WHEN** a commit subject contains spaces
- **THEN** the whole subject is captured, not just its first word

#### Scenario: Empty todo
- **WHEN** the todo file contains no instructions
- **THEN** the editor opens without error and writes back an equivalent file

### Requirement: Todo is presented as a structured table
In rebase mode the editor SHALL present the instructions as a table with one row per instruction,
rather than as free-form editable text.

#### Scenario: Table display
- **WHEN** a rebase todo is opened
- **THEN** each instruction appears as a row showing its action, commit reference, and subject

#### Scenario: Rows are visually differentiated by action
- **WHEN** instructions with different actions are displayed
- **THEN** the rows are colored to distinguish the actions from one another

### Requirement: Action cycling
The user SHALL be able to change the action on the selected instruction by cycling through the
available actions with a single key.

#### Scenario: Cycling through actions
- **WHEN** the user presses Tab on a selected instruction
- **THEN** its action advances to the next in the cycle of pick, squash, fixup, and drop

#### Scenario: Cycle wraps
- **WHEN** the action is the last in the cycle and the user presses Tab
- **THEN** it returns to the first

#### Scenario: Exec instructions do not cycle
- **WHEN** the selected instruction is an exec instruction
- **THEN** cycling leaves it unchanged, since it takes a command rather than a commit

### Requirement: Row navigation skips comments
The user SHALL be able to move the selection between instructions with the arrow keys, and comment
lines SHALL NOT be selectable.

#### Scenario: Moving the selection
- **WHEN** the user presses the down or up arrow
- **THEN** the selection moves to the next or previous instruction

#### Scenario: Comments are skipped
- **WHEN** a comment line sits between two instructions
- **THEN** moving the selection passes over it without stopping

#### Scenario: Selection stops at the ends
- **WHEN** the selection is on the first or last instruction and the user moves further in that
  direction
- **THEN** the selection stays where it is

### Requirement: Todo is written back in git's format
On save the editor SHALL write the todo back in exactly the format git parses, preserving line
order, comment lines, and every line it did not recognize.

#### Scenario: Round-trip without changes
- **WHEN** a todo file is opened and saved without changing any action
- **THEN** the file written is equivalent to the file read

#### Scenario: Round-trip after an action change
- **WHEN** the user changes an action and saves
- **THEN** only that action word differs, and line order, comments, and all other instructions are
  unchanged

#### Scenario: Abbreviated actions round-trip
- **WHEN** a todo written with abbreviated actions is saved
- **THEN** the result is a valid todo git parses to the same instructions

### Requirement: Long subjects wrap in the table
Commit subjects too wide for the subject column SHALL be wrapped across multiple lines rather than
truncated, so the full subject of every instruction is readable.

#### Scenario: Subject longer than the column
- **WHEN** a commit subject does not fit the available subject width
- **THEN** it is displayed across as many lines as it needs, with no text omitted

#### Scenario: Breaking at a word boundary
- **WHEN** a wrap point falls inside a word and a space exists earlier within the available width
- **THEN** the break is taken at that space rather than mid-word

#### Scenario: Word longer than the column
- **WHEN** a single word is itself wider than the available width
- **THEN** it is broken at the width, since no word boundary is available

#### Scenario: Subject that fits
- **WHEN** a subject fits within the available width
- **THEN** it is displayed on a single line, unchanged

### Requirement: Rows take the height their content needs
Table rows SHALL be sized to fit their wrapped subject, and scrolling SHALL account for rows of
differing heights so that selection and display stay aligned.

#### Scenario: Row height follows the wrap
- **WHEN** a subject wraps across several lines
- **THEN** its row occupies that many terminal lines

#### Scenario: Scrolling with mixed row heights
- **WHEN** the table contains both wrapped and unwrapped rows and the user moves the selection
- **THEN** the selected row remains visible and the rows stay correctly aligned with their content

#### Scenario: Very narrow terminal
- **WHEN** the terminal leaves no usable width for the subject column
- **THEN** the table still renders without error
