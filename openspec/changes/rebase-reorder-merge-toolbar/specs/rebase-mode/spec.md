## ADDED Requirements

### Requirement: Instructions can be reordered
The user SHALL be able to move the selected instruction earlier or later in the todo, changing the
order the commits are replayed in.

#### Scenario: Moving an instruction up
- **WHEN** the user presses Alt+Up on a selected instruction
- **THEN** it exchanges position with the instruction before it

#### Scenario: Moving an instruction down
- **WHEN** the user presses Alt+Down on a selected instruction
- **THEN** it exchanges position with the instruction after it

#### Scenario: Moving past a comment
- **WHEN** an instruction is moved past an interleaved comment line
- **THEN** the two instructions exchange position with each other
- **AND** the comment line stays where it was

#### Scenario: At the ends of the list
- **WHEN** the first instruction is moved up, or the last is moved down
- **THEN** the todo is left unchanged

### Requirement: Selection follows the moved instruction
After a move, the selection SHALL remain on the instruction that moved rather than on the position it
vacated, so that repeated presses move the same instruction.

#### Scenario: Repeated moves
- **WHEN** the user presses Alt+Up twice on the same instruction
- **THEN** that instruction has moved two positions earlier and is still selected

### Requirement: Reordering preserves the todo
Reordering SHALL NOT lose, duplicate, or alter any instruction, and SHALL preserve every comment line
and every unrecognized line.

#### Scenario: No instruction is lost
- **WHEN** instructions are reordered and the todo is saved
- **THEN** the saved file contains exactly the same instructions as before, in the new order

#### Scenario: Navigation stays correct after a move
- **WHEN** the user reorders instructions and then navigates with the arrow keys
- **THEN** the selection moves between instructions correctly, with no position skipped or repeated

### Requirement: Exec commands can be edited
The user SHALL be able to edit the command of an exec instruction from within the rebase table.

#### Scenario: Opening the command for editing
- **WHEN** the user presses Enter on a selected exec instruction
- **THEN** its command becomes editable in place

#### Scenario: Committing the edit
- **WHEN** the user presses Ctrl+S while editing an exec command
- **THEN** the edited command replaces the previous one and editing ends

#### Scenario: Discarding the edit
- **WHEN** the user presses Esc while editing an exec command
- **THEN** the command is left as it was and editing ends
- **AND** the editor does not exit

#### Scenario: Non-exec instructions
- **WHEN** the user presses Enter on an instruction that is not an exec
- **THEN** nothing happens

#### Scenario: Edited command is written back
- **WHEN** an exec command is edited and the todo is saved
- **THEN** the saved file contains the new command in git's exec instruction format
