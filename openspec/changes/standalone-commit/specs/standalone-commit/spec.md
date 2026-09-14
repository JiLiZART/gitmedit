## Purpose

Lets the editor be the entry point for making a commit — writing the message and submitting it —
rather than only being the editor git launches partway through its own commit flow.

## ADDED Requirements

### Requirement: Running with no arguments opens a commit editor
When invoked with no file argument inside a git repository, the editor SHALL open an editor for a new
commit message.

#### Scenario: Inside a repository with staged changes
- **WHEN** the user runs the editor with no arguments in a repository that has staged changes
- **THEN** an editor opens for writing the commit message

#### Scenario: Message area starts empty
- **WHEN** the standalone commit editor opens
- **THEN** the message area is empty and ready for typing

#### Scenario: Outside a repository
- **WHEN** the user runs the editor with no arguments outside any git repository
- **THEN** an error explaining that no repository was found is shown
- **AND** the editor does not open

#### Scenario: A file argument still behaves as before
- **WHEN** the editor is invoked with a file path
- **THEN** it edits that file exactly as it did before, with no standalone behavior

### Requirement: Nothing staged is reported before the editor opens
When there are no staged changes, the editor SHALL report that and exit before opening, so the user
never writes a message that cannot be committed.

#### Scenario: No staged changes
- **WHEN** the user runs the editor with no arguments and nothing is staged
- **THEN** a clear message saying there is nothing to commit is printed
- **AND** the editor does not open
- **AND** the process exits with a non-zero code

### Requirement: Saving submits the commit
On save, the editor SHALL write the message to a temporary file, invoke git to commit the staged
changes with that message, and exit with git's exit code.

#### Scenario: Successful commit
- **WHEN** the user writes a message and saves
- **THEN** the staged changes are committed with that message
- **AND** the process exits with the code git returned

#### Scenario: Terminal restored before git runs
- **WHEN** the commit is submitted
- **THEN** the terminal is restored to its normal state before git's output is produced, so that
  output is readable

#### Scenario: Temporary file cleaned up
- **WHEN** the commit attempt finishes, whether it succeeded or failed
- **THEN** the temporary message file no longer exists

### Requirement: Cancelling makes no commit
When the user cancels, the editor SHALL exit without committing anything and without modifying the
repository.

#### Scenario: Cancel before saving
- **WHEN** the user presses Esc in the standalone commit editor
- **THEN** no commit is created
- **AND** the staged changes remain staged

### Requirement: git's failure output is shown
When git refuses the commit, the editor SHALL show git's own output to the user before exiting,
rather than reporting a generic failure or discarding it.

#### Scenario: A commit hook rejects the message
- **WHEN** a commit hook rejects the commit
- **THEN** the hook's output is shown to the user
- **AND** the process exits with the code git returned

#### Scenario: Empty message
- **WHEN** the user saves without writing any message
- **THEN** the commit is not created
- **AND** the reason is shown to the user
