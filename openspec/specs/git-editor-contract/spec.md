# git-editor-contract Specification

## Purpose
Defines the process-level contract between gitmedit and git: how the file to edit is received, how
edits are written back, what the exit code communicates, and what state the terminal is left in.

## Requirements

### Requirement: File argument
The editor SHALL take the path of the file to edit from the first command-line argument when one is
given, and SHALL refuse to start when that path does not exist. The argument SHALL be optional:
invoked without one, the editor starts a commit as specified in `commit-without-arguments`.

#### Scenario: Path provided and exists
- **WHEN** the editor is invoked with the path of an existing file
- **THEN** the contents of that file are loaded for editing

#### Scenario: Path does not exist
- **WHEN** the editor is invoked with a path that does not exist
- **THEN** an error naming the missing path is written to stderr
- **AND** the process exits with code 1 without entering the editor

#### Scenario: No path given
- **WHEN** the editor is invoked with no arguments
- **THEN** it starts a commit rather than reporting a missing argument

#### Scenario: More than one argument
- **WHEN** the editor is invoked with more than one argument
- **THEN** a usage error is reported and the process exits with a non-zero code

### Requirement: Atomic write-back
On save, the editor SHALL write the content back to the same path it was given, atomically, so that
an interrupted write cannot leave the file truncated or partially written.

#### Scenario: Saving edited content
- **WHEN** the user saves after editing
- **THEN** the file at the original path contains exactly the content the editor assembled

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

### Requirement: Terminal is always restored
When the editor exits by any path, including an unexpected panic, it SHALL disable mouse capture,
leave the alternate screen, show the cursor, and disable raw mode, so the user returns to the shell
exactly as they left it. The cleanup SHALL NOT panic: errors during restoration are suppressed so
that restoration can still run while a panic is unwinding.

#### Scenario: Normal exit
- **WHEN** the editor exits after a save or a cancel
- **THEN** the terminal is out of raw mode, off the alternate screen, and not capturing the mouse
- **AND** the output shown before the editor started is visible again

#### Scenario: Panic during editing
- **WHEN** the editor panics while the terminal is set up
- **THEN** the terminal is fully restored before the panic message is printed
- **AND** the panic message is readable in the normal screen

#### Scenario: Restoration step fails
- **WHEN** one restoration step returns an error
- **THEN** the remaining steps still run and the process does not abort

### Requirement: Fast startup
The editor SHALL draw its first frame in well under 100 milliseconds on typical hardware, so that it
does not feel heavier than the nano or vim it replaces. Work that needs git beyond reading the comment
character, such as loading rebase commit details, SHALL NOT delay the first frame.

#### Scenario: Cold invocation
- **WHEN** the editor binary is invoked
- **THEN** the time from process start to a drawn first frame stays under 100 milliseconds

#### Scenario: Rebase todo with many commits
- **WHEN** a rebase todo listing many commits is opened
- **THEN** the first frame is drawn without waiting for any commit details to load
