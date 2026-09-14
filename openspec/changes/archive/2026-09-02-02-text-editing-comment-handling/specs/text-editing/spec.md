## Purpose

The editing surface for git message files: how text is entered and manipulated, and how the lines
git owns are told apart from the lines the user owns and preserved unchanged.

## ADDED Requirements

### Requirement: Full text editing
The editor SHALL support the ordinary text editing operations: inserting and deleting characters at
any position, moving the cursor with the arrow keys, jumping to the start and end of a line, and
creating new lines for multiline messages.

#### Scenario: Inserting text
- **WHEN** the user types with the cursor positioned inside a line
- **THEN** the typed characters appear at the cursor position

#### Scenario: Moving the cursor
- **WHEN** the user presses an arrow key
- **THEN** the cursor moves one position in that direction, within the bounds of the text

#### Scenario: Jumping to line boundaries
- **WHEN** the user presses Home or End
- **THEN** the cursor moves to the start or the end of the current line

#### Scenario: Creating a multiline message
- **WHEN** the user presses Enter
- **THEN** a new line is created and subsequent typing continues on it

### Requirement: Nano-style editing shortcuts
The editor SHALL provide nano-style control shortcuts for saving, cancelling, deleting a line,
deleting a word, and undoing or redoing an edit.

#### Scenario: Save
- **WHEN** the user presses Ctrl+S
- **THEN** the edited content is written back and the editor exits successfully

#### Scenario: Cancel
- **WHEN** the user presses Esc
- **THEN** the editor exits without modifying the file

#### Scenario: Delete the current line
- **WHEN** the user presses Ctrl+U
- **THEN** the content of the current line is removed

#### Scenario: Delete a word
- **WHEN** the user presses Ctrl+W or Ctrl+D
- **THEN** the previous or next word is deleted respectively

#### Scenario: Undo and redo
- **WHEN** the user presses Ctrl+Z after making an edit
- **THEN** that edit is reverted
- **AND** pressing Ctrl+Y reapplies it

### Requirement: System clipboard
The editor SHALL copy, cut, and paste through the system clipboard, and SHALL continue working
without error where no system clipboard is available, such as over SSH or on a headless machine.

#### Scenario: Copy and paste
- **WHEN** the user selects text and presses Ctrl+C, then presses Ctrl+V elsewhere
- **THEN** the copied text is inserted at the new cursor position

#### Scenario: Cut
- **WHEN** the user selects text and presses Ctrl+X
- **THEN** the text is removed from the message and placed on the system clipboard

#### Scenario: No clipboard available
- **WHEN** a clipboard operation is attempted and the system provides no clipboard
- **THEN** the operation does nothing and the editor keeps running without an error

### Requirement: Line classification
On load, the editor SHALL classify every line of the file as content, as a comment, or as a conflict
marker, using the comment character git is configured to use.

#### Scenario: Comment line
- **WHEN** a line begins with the configured comment character
- **THEN** it is classified as a comment

#### Scenario: Conflict marker line
- **WHEN** a line begins with a run of `<`, `=`, or `>` marker characters
- **THEN** it is classified as a conflict marker

#### Scenario: Ordinary line
- **WHEN** a line begins with anything else, including an empty line
- **THEN** it is classified as content

### Requirement: Protected lines are not editable
Lines that belong to git rather than to the user — comments and conflict markers — SHALL be displayed
but SHALL NOT be editable, so the user cannot damage the structure git expects.

#### Scenario: Cursor skips protected lines
- **WHEN** the user moves the cursor through a message containing comment lines
- **THEN** the cursor visits only content lines

#### Scenario: Protected lines remain visible
- **WHEN** a message contains comments or conflict markers
- **THEN** those lines are shown in the editor, visually distinct from editable content

### Requirement: Protected lines are preserved byte for byte
On save, the editor SHALL reproduce every protected line exactly as it was read, in its original
position, so that a file which round-trips without edits is unchanged.

#### Scenario: Untouched file round-trips unchanged
- **WHEN** a file is opened and saved without any edit
- **THEN** the bytes written are identical to the bytes read

#### Scenario: Edited file keeps its comments
- **WHEN** the user edits content lines and saves
- **THEN** every comment and conflict-marker line is written back unchanged and in its original
  position, interleaved with the edited content

### Requirement: Long lines scroll horizontally
When a line is wider than the terminal, the editor SHALL scroll the view horizontally to keep the
cursor visible rather than reflowing the line, so that the stored line structure always matches what
will be written back.

#### Scenario: Cursor moves beyond the right edge
- **WHEN** the cursor moves past the right edge of the terminal
- **THEN** the view scrolls horizontally so the cursor stays visible

#### Scenario: Line structure is not changed by display
- **WHEN** a line too long for the terminal is displayed and then saved without editing
- **THEN** it is written back as a single line, with no break introduced by the display
