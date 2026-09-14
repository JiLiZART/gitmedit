# text-editing Specification

## Purpose
The message editor in the left pane: the part of a commit, merge, tag, or squash message file that the
user writes. It covers how the file is split into the user's message and git's comment trailer, how
the file is reassembled on save, and the editing behavior of the text area itself.

## Requirements

### Requirement: Message files are split into message and trailer
When a message file is opened (commit, merge, tag, or squash), the editor SHALL split it into the
**message**, which is shown in the editor, and the **trailer**, which is kept aside for the status
pane and for write-back. Comment lines are recognized by git's configured comment character.

- The trailer begins at the first comment line and runs to the end of the file. A scissors line
  (`<comment char> ------------------------ >8 ------------------------`) and everything below it
  always belong to the trailer, including uncommented diff lines.
- The message is every line before the trailer, with trailing blank lines removed.
- If non-blank, non-comment lines appear inside the trailer above any scissors line, they SHALL be
  moved into the message in their original order. The trailer then keeps only its comment lines, its
  blank lines, and the scissors section. Blank lines left adjacent by the move SHALL be collapsed to
  one.

#### Scenario: Commit message with a status block
- **WHEN** `fixtures/ammend2_fixture.txt` is opened
- **THEN** the editor shows the single line `fix: release volume`
- **AND** no comment line is shown in the editor

#### Scenario: Fresh commit with no message yet
- **WHEN** a commit message file that begins with a blank line followed by comments is opened
- **THEN** the editor is empty with the cursor at the start

#### Scenario: Merge message with separated comment groups
- **WHEN** `fixtures/merge_fixture.txt` is opened
- **THEN** the editor shows only the `Merge remote-tracking branch ...` line
- **AND** the blank lines between the comment groups stay in the trailer

#### Scenario: Squash message with interleaved commit messages
- **WHEN** a squash message is opened in which each combined commit's message sits under its own
  `This is the commit message #n:` comment
- **THEN** the editor shows every combined message in order, separated by single blank lines
- **AND** the numbering comments are in the trailer

#### Scenario: Verbose commit diff
- **WHEN** a commit message file contains a scissors line followed by a diff
- **THEN** neither the scissors line nor the diff is shown in the editor

### Requirement: Save reassembles the file
On save, the editor SHALL write the message with trailing blank lines removed, then exactly one blank
line, then the trailer exactly as it was kept. When the message is empty, it SHALL write a single blank
line followed by the trailer. When there is no trailer, it SHALL write the message alone. The original
file's final-newline state SHALL be preserved.

#### Scenario: Untouched git-generated file round-trips unchanged
- **WHEN** any file in `fixtures/` other than the rebase todo is opened and saved without edits
- **THEN** the bytes written are identical to the bytes read

#### Scenario: Edited message keeps the trailer
- **WHEN** the user rewrites the message and saves
- **THEN** the file contains the new message, one blank line, and the original trailer unchanged

#### Scenario: Interleaved file saved without edits
- **WHEN** a file whose message lines were moved out of the trailer is saved without edits
- **THEN** the file contains the message, one blank line, and every trailer line in original order
- **AND** git's default comment stripping yields the same commit message as the original file

#### Scenario: Unrecognized file has no split
- **WHEN** the editor is invoked on a file that is not a known git message file
- **THEN** the whole file is shown in the editor, comment lines included, and saved exactly as edited

### Requirement: Full text editing
The editor SHALL support the ordinary text editing operations: inserting and deleting characters at
any position, moving the cursor with the arrow keys, jumping to the start and end of a line, and
creating new lines for multiline messages. Every line of the message SHALL be reachable and editable.

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
- **THEN** the file is written back and the editor exits successfully

#### Scenario: Cancel
- **WHEN** the message pane has focus and the user presses Esc
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
The editor SHALL copy, cut, and paste through the system clipboard, and SHALL keep working without
error where no system clipboard is available, such as over SSH or on a headless machine.

#### Scenario: Copy and paste
- **WHEN** the user selects text and presses Ctrl+C, then presses Ctrl+V elsewhere
- **THEN** the copied text is inserted at the new cursor position

#### Scenario: Cut
- **WHEN** the user selects text and presses Ctrl+X
- **THEN** the text is removed from the message and placed on the system clipboard

#### Scenario: No clipboard available
- **WHEN** a clipboard operation is attempted and the system provides no clipboard
- **THEN** the operation does nothing and the editor keeps running without an error

### Requirement: Long lines soft-wrap
Lines wider than the message pane SHALL be wrapped for display at the pane width, breaking at word
boundaries where possible. Wrapping SHALL be display only: it never inserts line breaks into the
message, and the cursor moves through wrapped lines by display row.

#### Scenario: Long body line
- **WHEN** a message line is wider than the pane
- **THEN** it is displayed across as many rows as it needs, with no text hidden and no horizontal
  scrolling

#### Scenario: Wrapping does not change the file
- **WHEN** a wrapped line is saved without editing
- **THEN** it is written back as a single line

#### Scenario: Cursor moves by display row
- **WHEN** the cursor is on the first display row of a wrapped line and the user presses Down
- **THEN** the cursor moves to the next display row of the same line

#### Scenario: Word wider than the pane
- **WHEN** a single word is wider than the pane
- **THEN** it is broken at the pane width

### Requirement: Conflict markers are highlighted
Lines in the message that begin with a run of six or seven `<`, `=`, or `>` characters SHALL be
displayed in a style clearly distinct from ordinary text. They SHALL remain ordinary editable lines.

#### Scenario: Marker lines styled
- **WHEN** a message containing `<<<<<<<`, `=======`, and `>>>>>>>` lines is opened
- **THEN** those lines are rendered in the conflict-marker style

#### Scenario: Markers can be removed
- **WHEN** the user deletes the marker lines and saves
- **THEN** the file written back no longer contains them

### Requirement: No content-derived feedback while typing
The message editor SHALL NOT show counters, warnings, or other feedback computed from the message
text. The key bar shows keys only.

#### Scenario: Typing a long subject
- **WHEN** the user types a subject line longer than 72 characters
- **THEN** no counter or warning appears
