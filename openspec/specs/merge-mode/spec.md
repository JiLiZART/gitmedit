# merge-mode Specification

## Purpose
Handles editing a merge message, where the file may contain conflict markers that carry structural
meaning and must be shown clearly to the person resolving the merge.

## Requirements

### Requirement: Conflict markers are detected and styled
When editing a merge message, the editor SHALL detect conflict markers and display them in a
visually distinct style, so the boundaries of a conflict are obvious at a glance.

#### Scenario: Markers are highlighted
- **WHEN** a merge message containing `<<<<<<`, `======`, or `>>>>>>` lines is opened
- **THEN** each of those lines is rendered in a style clearly distinct from ordinary message text

#### Scenario: Marker length variation
- **WHEN** markers appear with the six or seven character forms git emits
- **THEN** they are detected as conflict markers in either form

### Requirement: The resolved message is editable
The message text between and around conflict markers SHALL be fully editable, so the user can write
the resolution without leaving the editor.

#### Scenario: Editing between markers
- **WHEN** the user places the cursor on a content line inside a conflict block and types
- **THEN** the text is edited normally and is written back on save

### Requirement: Conflict markers are editable
Conflict-marker lines in a merge message SHALL be editable, so the user can remove them as part of
resolving the conflict. Comment lines SHALL remain protected and SHALL be written back unchanged.

#### Scenario: Markers can be edited
- **WHEN** the user moves the cursor onto a conflict-marker line in a merge message
- **THEN** the cursor enters the line and it can be edited

#### Scenario: Markers can be removed
- **WHEN** the user deletes the conflict-marker lines while resolving and saves
- **THEN** the file written back no longer contains them

#### Scenario: Comments still survive a save
- **WHEN** the user edits a merge message and saves
- **THEN** every comment line is written back unchanged and in its original position
