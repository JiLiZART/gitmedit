## MODIFIED Requirements

### Requirement: File argument
The editor SHALL take the path of the file to edit from the first command-line argument when one is
given, and SHALL refuse to start when that path does not exist. The argument SHALL be optional:
invoked without one, the editor enters standalone commit mode rather than reporting a usage error.

#### Scenario: Path provided and exists
- **WHEN** the editor is invoked with the path of an existing file
- **THEN** the contents of that file are loaded for editing

#### Scenario: Path does not exist
- **WHEN** the editor is invoked with a path that does not exist
- **THEN** an error naming the missing path is written to stderr
- **AND** the process exits with code 1 without entering the editor

#### Scenario: No path given
- **WHEN** the editor is invoked with no arguments
- **THEN** it enters standalone commit mode rather than reporting a missing argument

#### Scenario: More than one path given
- **WHEN** the editor is invoked with more than one argument
- **THEN** a usage error is reported and the process exits with a non-zero code
