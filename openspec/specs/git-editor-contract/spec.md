# git-editor-contract Specification

## Purpose
Defines the process-level contract between gitmedit and git: how the file to edit is received, how
edits are written back, what the exit code communicates, and what state the terminal is left in.

## Requirements

### Requirement: File argument
The editor SHALL take the path of the file to edit from the first command-line argument, and SHALL
refuse to start when that path does not exist.

#### Scenario: Path provided and exists
- **WHEN** the editor is invoked with the path of an existing file
- **THEN** the contents of that file are loaded for editing

#### Scenario: Path does not exist
- **WHEN** the editor is invoked with a path that does not exist
- **THEN** an error naming the missing path is written to stderr
- **AND** the process exits with code 1 without entering the editor

### Requirement: Atomic write-back
On save, the editor SHALL write the edited content back to the same path it was given, and SHALL do
so atomically so that an interrupted write cannot leave the file truncated or partially written.

#### Scenario: Saving edited content
- **WHEN** the user saves after editing
- **THEN** the file at the original path contains exactly the edited content

#### Scenario: No temporary files left behind
- **WHEN** a save completes successfully
- **THEN** no temporary or intermediate file remains beside the target file

#### Scenario: Line endings preserved
- **WHEN** content is written back
- **THEN** unix line endings are emitted and no additional trailing content is introduced

### Requirement: Exit codes signal intent
The editor SHALL exit with code 0 when the user saves, and with code 1 when the user cancels or an
error occurs, so that git can distinguish acceptance from abort.

#### Scenario: User saves
- **WHEN** the user saves
- **THEN** the process exits with code 0

#### Scenario: User cancels
- **WHEN** the user cancels without saving
- **THEN** the file on disk is left unmodified
- **AND** the process exits with code 1

### Requirement: Terminal raw mode is always restored
The editor SHALL restore the terminal out of raw mode when it exits, on every path including an
unexpected panic, so the user is never returned to a shell that does not echo input.

#### Scenario: Normal exit
- **WHEN** the editor exits after a save or a cancel
- **THEN** the terminal is no longer in raw mode

#### Scenario: Panic during editing
- **WHEN** the editor panics while the terminal is in raw mode
- **THEN** raw mode is disabled before the panic message is printed
- **AND** the panic message is still shown to the user

### Requirement: Fast startup
The editor SHALL start in well under 100 milliseconds on typical hardware, so that it does not feel
heavier than the nano or vim it replaces.

#### Scenario: Cold invocation
- **WHEN** the editor binary is invoked
- **THEN** the time from process start to a drawn first frame stays under 100 milliseconds
