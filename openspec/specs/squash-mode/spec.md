# squash-mode Specification

## Purpose
Supports writing the combined message for a squash, keeping the log of the commits being combined
visible as reference while making clear that only the message below it is editable.

## Requirements

### Requirement: Squash operation is recognized
The editor SHALL recognize when it has been handed a squash message and SHALL separate the generated
log header from the message the user is expected to write.

#### Scenario: Squash message file
- **WHEN** the editor is invoked on a squash message file
- **THEN** the leading generated header is identified and separated from the message body

#### Scenario: File with no recognizable header
- **WHEN** the file contains no identifiable header block
- **THEN** the whole file is treated as an editable message rather than failing

### Requirement: Commit log is shown read-only
The log of the commits being combined SHALL be displayed as context and SHALL NOT be editable.

#### Scenario: Log is visible
- **WHEN** a squash message is opened
- **THEN** the commit log is displayed in its own area, labelled as read-only

#### Scenario: Log cannot be edited
- **WHEN** the user types
- **THEN** the log area is unaffected and the text goes to the message area

### Requirement: Combined message is editable
The message portion of a squash file SHALL be fully editable using the ordinary text editing
behavior.

#### Scenario: Writing the combined message
- **WHEN** the user edits the message area and saves
- **THEN** the edited message is written back

### Requirement: Generated content is preserved on save
On save the editor SHALL write the log header back exactly as it was read, ahead of the edited
message, so that the file git reads keeps the structure git produced.

#### Scenario: Header survives editing
- **WHEN** the user edits the message and saves
- **THEN** the header lines are written back unchanged, in their original order, before the message

#### Scenario: Protected lines inside the message
- **WHEN** the message portion contains comment lines
- **THEN** those lines are preserved byte for byte on save
