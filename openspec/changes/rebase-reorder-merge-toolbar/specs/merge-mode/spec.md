## ADDED Requirements

### Requirement: Conflicted files are surfaced
In merge mode the editor SHALL parse the list of conflicted files from the comment block git
generated in the message, and SHALL display it, so the user can see what conflicted without reading
through the comment text.

#### Scenario: Message lists conflicts
- **WHEN** a merge message contains git's conflicted-files block
- **THEN** the conflicted file names are displayed

#### Scenario: No conflict block present
- **WHEN** a merge message contains no conflicted-files block
- **THEN** nothing is displayed in its place, with no empty heading or placeholder

#### Scenario: More files than fit
- **WHEN** the conflicted file list is longer than the space available
- **THEN** the display degrades without overflowing into the content area

#### Scenario: Display does not affect the file
- **WHEN** a merge message whose conflicts were displayed is saved
- **THEN** the comment block is written back unchanged, exactly as any other comment
