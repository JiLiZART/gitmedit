## ADDED Requirements

### Requirement: Editing mode follows the git operation
The editor SHALL select an editing mode when the file is opened, based on the git operation
detected, and that mode SHALL determine which kinds of line are editable. Commit, tag, and
unrecognized files SHALL use a plain mode in which every line is editable.

#### Scenario: Commit message is fully editable
- **WHEN** the user opens a commit message and moves the cursor onto a comment line
- **THEN** the cursor enters that line and typing replaces its content

#### Scenario: Deleting git's instruction block
- **WHEN** the user deletes the comment lines git prepended to a commit message and saves
- **THEN** the file written back no longer contains them

#### Scenario: Unrecognized file
- **WHEN** the editor is invoked on a file that is not one of the known git files
- **THEN** every line is editable, as in an ordinary text editor

#### Scenario: Merge mode keeps comments protected
- **WHEN** the user opens a merge message
- **THEN** comment lines remain protected while other lines are editable

#### Scenario: Squash mode is unchanged
- **WHEN** the user opens a squash message
- **THEN** comments and the generated log remain protected, as before

### Requirement: Editing uses the text area's default behavior
Editing SHALL follow the default behavior of the underlying text editing surface, without
special-cased handling that skips or reinterprets particular lines.

#### Scenario: Cursor movement is uniform
- **WHEN** the user moves the cursor through a message in plain mode
- **THEN** every line is reachable, with no line skipped or treated specially

#### Scenario: No editing feedback in the status bar
- **WHEN** the user edits a commit message
- **THEN** the status bar shows the available keys and nothing derived from the message content

## MODIFIED Requirements

### Requirement: Protected lines are not editable
In the editing modes that protect lines, the lines that belong to git rather than to the user SHALL
be displayed but SHALL NOT be editable, so the user cannot damage the structure git expects. Which
lines are protected depends on the mode: merge protects comments, and squash protects both comments
and conflict markers. In plain mode no line is protected.

#### Scenario: Cursor skips protected lines
- **WHEN** the user moves the cursor through a message in a mode that protects lines
- **THEN** the cursor visits only the lines that mode leaves editable

#### Scenario: Protected lines remain visible
- **WHEN** a message contains comments or conflict markers
- **THEN** those lines are shown in the editor, visually distinct from editable content

#### Scenario: Plain mode protects nothing
- **WHEN** the message is opened in plain mode
- **THEN** every line is reachable and editable

### Requirement: Protected lines are preserved byte for byte
On save, the editor SHALL reproduce every protected line exactly as it was read, in its original
position. In plain mode, where no line is protected, the file written back is exactly the content of
the editing surface.

#### Scenario: Untouched file round-trips unchanged
- **WHEN** a file is opened and saved without any edit
- **THEN** the bytes written are identical to the bytes read

#### Scenario: Edited file keeps its comments
- **WHEN** the user edits content lines in a mode that protects comments and saves
- **THEN** every protected line is written back unchanged and in its original position, interleaved
  with the edited content

#### Scenario: Plain mode writes what the user sees
- **WHEN** the user edits a message in plain mode and saves
- **THEN** the file contains exactly the lines shown in the editor, including any edited or deleted
  comment lines
