# text-editing Specification

## Purpose
The editing surface for git message files: how text is entered and manipulated, and how the lines
git owns are told apart from the lines the user owns and preserved unchanged.

## Requirements

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
