# rebase-mode Specification

## Purpose
Presents an interactive rebase todo as a table of instructions in the left pane and the selected
commit's details in the right pane. The user can change actions, reorder commits, and reword subjects
without hand-editing a file that git will refuse if it is malformed, and can drop to raw text for
anything the table does not express.

## Requirements

### Requirement: Todo file is parsed into typed lines
The editor SHALL parse a rebase todo into typed lines:

- **commit instructions**: an action (pick, reword, edit, squash, fixup, drop, in long or one-letter
  form), optional action flags such as `fixup -C`, a commit reference, and the rest of the line
- **other instructions**: exec, break, label, reset, merge, and update-ref, each with the rest of its
  line kept verbatim
- **comment lines**, **blank lines**, and **unrecognized lines**

#### Scenario: Instruction line
- **WHEN** a line reads `pick 54763e6 # docs(state): record phase 8 context session`
- **THEN** it is parsed as a pick of `54763e6` with the rest of the line recorded as its subject

#### Scenario: Abbreviated actions
- **WHEN** an instruction uses git's one-letter abbreviation
- **THEN** it is parsed as the same action as the long form

#### Scenario: Comment line
- **WHEN** a line begins with the configured comment character
- **THEN** it is preserved as a comment and not treated as an instruction

#### Scenario: Subject containing spaces
- **WHEN** a commit subject contains spaces
- **THEN** the whole subject is captured, not just its first word

#### Scenario: Fixup with a flag
- **WHEN** a line reads `fixup -C 7314ba6 subject`
- **THEN** it is parsed as a fixup carrying the `-C` flag

#### Scenario: Unrecognized line
- **WHEN** a line matches none of the forms
- **THEN** it is kept unchanged rather than discarded or reinterpreted

#### Scenario: Empty todo
- **WHEN** the todo file contains no instructions
- **THEN** the editor opens without error and writes back an equivalent file

### Requirement: Todo is presented as a structured table
The left pane SHALL show one row per instruction, with columns for action, commit reference, and
subject. Rows SHALL be colored by action. Comment and blank lines SHALL NOT be shown in the table;
git's command legend is available in the shortcut reference instead. Unrecognized lines SHALL be shown
as dimmed, non-selectable rows in their original positions.

#### Scenario: Table display
- **WHEN** `fixtures/squash_fixture.txt` is opened
- **THEN** the table shows 14 rows, each with its action, commit reference, and subject
- **AND** the pane title shows the rebase range `36d7eda..aa619f8`

#### Scenario: Rows are visually differentiated by action
- **WHEN** instructions with different actions are displayed
- **THEN** each action has its own color

### Requirement: Long subjects wrap and rows size to fit
Subjects wider than the subject column SHALL wrap at word boundaries, breaking mid-word only when a
word is wider than the column. Each row SHALL be as tall as its wrapped subject, and scrolling SHALL
keep the selected row fully visible when rows differ in height.

#### Scenario: Subject longer than the column
- **WHEN** a subject does not fit the subject column
- **THEN** it is displayed across as many lines as it needs, with no text omitted

#### Scenario: Scrolling with mixed row heights
- **WHEN** the table has wrapped and unwrapped rows and the selection moves past the visible area
- **THEN** the selected row is fully visible and rows stay aligned with their content

#### Scenario: Very narrow pane
- **WHEN** the pane leaves no usable width for the subject column
- **THEN** the table still renders without error

### Requirement: Selection moves between instructions
Up and Down SHALL move the selection to the previous or next selectable row, and Home and End to the
first and last. Unrecognized rows are not selectable. The selection SHALL stop at the ends.

#### Scenario: Moving the selection
- **WHEN** the user presses Down
- **THEN** the selection moves to the next instruction row

#### Scenario: Selection stops at the ends
- **WHEN** the selection is on the last instruction and the user presses Down
- **THEN** the selection stays where it is

### Requirement: Actions are set by letter or cycled
On a selected commit instruction, the keys `p`, `r`, `e`, `s`, `f`, and `d` SHALL set the action to
pick, reword, edit, squash, fixup, and drop. Tab SHALL cycle through pick, squash, fixup, and drop,
wrapping from the last back to the first; when the current action is outside that cycle, Tab SHALL set
it to squash. Changing the action away from fixup SHALL drop any fixup flag. On other instructions,
action keys SHALL do nothing.

#### Scenario: Setting an action directly
- **WHEN** the user presses `f` on a selected pick instruction
- **THEN** its action becomes fixup

#### Scenario: Cycling through actions
- **WHEN** the user presses Tab on a pick instruction
- **THEN** its action becomes squash, and further presses give fixup, drop, then pick again

#### Scenario: Cycling from reword
- **WHEN** the user presses Tab on a reword instruction
- **THEN** its action becomes squash

#### Scenario: Exec instructions are unaffected
- **WHEN** the selected row is an exec instruction and the user presses `s` or Tab
- **THEN** the instruction is unchanged

### Requirement: Instructions can be reordered
Alt+Up and Alt+Down SHALL move the selected instruction one instruction up or down. The selection
SHALL follow the moved instruction. Comment, blank, and unrecognized lines SHALL keep their positions
in the file; only instructions trade places among the instruction slots. Moving past the first or last
instruction SHALL do nothing.

#### Scenario: Moving a commit up
- **WHEN** the third instruction is selected and the user presses Alt+Up
- **THEN** it becomes the second instruction, the former second becomes third, and the selection is
  on the moved instruction

#### Scenario: Comments stay in place
- **WHEN** a todo reads pick A, a comment, pick B, and the user moves B up
- **THEN** the written file reads pick B, the comment, pick A

#### Scenario: Nothing lost or duplicated
- **WHEN** the user performs any sequence of moves and saves
- **THEN** the written todo contains every original instruction exactly once

### Requirement: Subjects can be reworded inline
Enter on a selected commit instruction SHALL open its subject for inline editing. Enter SHALL confirm
the edit and set the action to reword. Esc SHALL discard the edit and leave the row unchanged. While
the inline editor is open, action, reorder, and focus keys SHALL apply to the text instead. A row with
a pending reword SHALL be marked visibly and show the new subject.

#### Scenario: Rewording a subject
- **WHEN** the user presses Enter on a pick row, types a new subject, and presses Enter
- **THEN** the row shows the new subject, is marked as pending reword, and its action is reword

#### Scenario: Discarding an inline edit
- **WHEN** the user edits a subject and presses Esc
- **THEN** the row shows its original subject and action

#### Scenario: Changing action after rewording
- **WHEN** a row with a pending reword has its action changed to anything other than reword
- **THEN** the pending reword is discarded and the original subject is shown

### Requirement: Pending rewords are applied when git asks for the message
On save, for each instruction with a pending reword, the editor SHALL store the new commit message in
`<gitdir>/gitmedit/reword/<commit>`: the new subject followed by the commit's original body, which is
read with git. If the body cannot be read, only the subject SHALL be stored. When the editor is later
invoked on `COMMIT_EDITMSG` during that rebase, and the last line of `<gitdir>/rebase-merge/done` is a
reword of a commit that has a stored message, the editor SHALL use the stored message as the message
in place of the file's own, keeping the file's trailer. The stored message SHALL be deleted after that
file is saved. Stored messages from an earlier rebase SHALL be deleted when a new todo is opened. The
todo line itself SHALL be written with its original subject text.

#### Scenario: Reword flows through to the commit
- **WHEN** the user rewords a subject, saves the todo, and git stops to reword that commit
- **THEN** the message editor opens already containing the new subject and the original body
- **AND** saving it produces a commit with that message and removes the stored message

#### Scenario: Todo line format
- **WHEN** a todo with a pending reword is saved
- **THEN** that line differs from the original only in its action word

#### Scenario: Cancelled todo stores nothing
- **WHEN** the user rewords a subject and then cancels
- **THEN** no stored message is written

#### Scenario: Message editor is not gitmedit
- **WHEN** git's message editor is a different program
- **THEN** git opens that editor with the original message, as a normal reword

#### Scenario: Stale stored messages
- **WHEN** a new rebase todo is opened and stored messages from an earlier rebase exist
- **THEN** they are deleted

### Requirement: Raw text toggle
Ctrl+E SHALL switch the left pane between the table and a plain text editor containing the todo exactly
as it would be written now, comments included. Switching back SHALL re-parse the text into the table.
Pending rewords SHALL survive the round trip for every commit that is still present with the reword
action, and be discarded otherwise. Saving from the raw editor SHALL write the text as shown.

#### Scenario: Adding an exec line
- **WHEN** the user presses Ctrl+E, adds `exec cargo test` after an instruction, and presses Ctrl+E
- **THEN** the table shows an exec row at that position

#### Scenario: Raw text shows current state
- **WHEN** the user reorders and changes actions in the table and then presses Ctrl+E
- **THEN** the raw text reflects those changes

#### Scenario: Unparseable edits are kept
- **WHEN** the user types a line in the raw editor that matches no instruction form and switches back
- **THEN** the line is shown as an unrecognized row and written back unchanged

### Requirement: Right pane shows the result and the selected commit
The right pane SHALL show a result summary at the top, followed by the selected commit's details.

- **Summary**: computed from the table without running git. It shows the resulting number of commits
  (pick, reword, and edit each count as one; squash, fixup, and drop count as none), the counts of
  squash/fixup, drop, and reword, and a warning when the first commit instruction is a squash or fixup.
- **Details**: the selected commit's full message and a stat of the files it changed, read with git
  when the selection first lands on that commit and cached by commit. Details SHALL NOT block startup
  or input. If git fails, the pane SHALL say the details are unavailable.

The pane scrolls as the status pane does.

#### Scenario: Summary updates with edits
- **WHEN** a 14-commit todo has two instructions set to squash and one to drop
- **THEN** the summary reads 14 → 11 commits with 2 squash/fixup and 1 drop

#### Scenario: Invalid first squash
- **WHEN** the first commit instruction is set to squash
- **THEN** the summary shows a warning that the first commit cannot be squashed

#### Scenario: Details for the selection
- **WHEN** the selection lands on a commit
- **THEN** the right pane shows that commit's full message and its changed files with badges

#### Scenario: Fast navigation
- **WHEN** the user holds Down through many commits
- **THEN** the table responds immediately and details fill in as they load

#### Scenario: Git unavailable
- **WHEN** git cannot be run or does not know the commit
- **THEN** the details area reads "details unavailable" and the editor keeps working

### Requirement: Todo is written back in git's format
On save the editor SHALL write the todo in exactly the format git parses, preserving comment lines,
blank lines, unrecognized lines, each instruction's text after its commit reference, and action flags.
When the file is saved without changes, the output SHALL match the input.

#### Scenario: Round-trip without changes
- **WHEN** a todo file is opened and saved without changes
- **THEN** the bytes written are identical to the bytes read

#### Scenario: Round-trip after an action change
- **WHEN** the user changes one action and saves
- **THEN** only that action word differs from the original

#### Scenario: Abbreviated actions round-trip
- **WHEN** a todo written with abbreviated actions is saved after changing a different line
- **THEN** the unchanged lines keep their abbreviations
