## Purpose

Handles editing a merge message, where the file may contain conflict markers that carry structural
meaning and must be shown clearly to the person resolving the merge.

## ADDED Requirements

### Requirement: Conflict markers are detected and styled
When editing a merge message, the editor SHALL detect conflict markers and display them in a
visually distinct style, so the boundaries of a conflict are obvious at a glance.

#### Scenario: Markers are highlighted
- **WHEN** a merge message containing `<<<<<<`, `======`, or `>>>>>>` lines is opened
- **THEN** each of those lines is rendered in a style clearly distinct from ordinary message text

#### Scenario: Marker length variation
- **WHEN** markers appear with the six or seven character forms git emits
- **THEN** they are detected as conflict markers in either form

### Requirement: Conflict markers are protected
Conflict-marker lines SHALL NOT be editable, and SHALL be written back exactly as they were read.

#### Scenario: Markers cannot be edited
- **WHEN** the user moves through a merge message
- **THEN** the cursor does not enter conflict-marker lines

#### Scenario: Markers survive a save
- **WHEN** the user edits the message around the markers and saves
- **THEN** every marker line is written back unchanged and in its original position

### Requirement: The resolved message is editable
The message text between and around conflict markers SHALL be fully editable, so the user can write
the resolution without leaving the editor.

#### Scenario: Editing between markers
- **WHEN** the user places the cursor on a content line inside a conflict block and types
- **THEN** the text is edited normally and is written back on save
