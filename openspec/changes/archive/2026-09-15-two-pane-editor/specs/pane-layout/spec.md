## Purpose

Arranges every editing session as two panes: an editing pane on the left, where the user changes what
git will read, and a context pane on the right, where the information git put in the file is shown in
a readable form. Covers screen ownership, focus, mouse and keyboard scrolling, narrow terminals, and
the key bar.

## ADDED Requirements

### Requirement: Full-screen two-pane layout
The editor SHALL draw on the terminal's alternate screen and SHALL divide it into a left editing pane
and a right context pane placed side by side, above a single-line key bar. The left pane SHALL take
roughly 60% of the width and the right pane the rest. Each pane SHALL have a border carrying a title.

#### Scenario: Message file layout
- **WHEN** a commit, merge, tag, or squash message is opened in a wide terminal
- **THEN** the left pane shows the message editor titled with the file name
- **AND** the right pane shows the status pane titled with the current branch when known, or
  "Status" otherwise

#### Scenario: Rebase todo layout
- **WHEN** a rebase todo is opened in a wide terminal
- **THEN** the left pane shows the rebase table titled with the rebase range
- **AND** the right pane shows the details of the selected commit

#### Scenario: Nothing to show on the right
- **WHEN** the opened file yields no context, such as an unrecognized file or a message with no
  comment block
- **THEN** the right pane is not drawn and the left pane takes the full width

#### Scenario: Previous terminal content restored on exit
- **WHEN** the editor exits by any path
- **THEN** the alternate screen is left and the terminal shows what it showed before the editor
  started

### Requirement: Resize redraws the layout
The editor SHALL recompute the layout whenever the terminal is resized, without losing edits, focus,
selection, or scroll position.

#### Scenario: Terminal resized while editing
- **WHEN** the terminal is resized while the editor is open
- **THEN** both panes redraw at the new size and the message content is unchanged

#### Scenario: Resize crosses the narrow threshold
- **WHEN** a resize moves the width across the narrow-terminal threshold
- **THEN** the layout switches between side-by-side and single-pane presentation accordingly

### Requirement: One pane has focus
Exactly one pane SHALL have focus at a time. Keystrokes other than the global shortcuts SHALL go only
to the focused pane. The focused pane SHALL be indicated by a highlighted border. The left pane SHALL
have focus when the editor opens.

#### Scenario: Initial focus
- **WHEN** the editor opens
- **THEN** the left pane has focus and its border is highlighted

#### Scenario: Typing with the right pane focused
- **WHEN** the right pane has focus and the user types ordinary characters
- **THEN** the message and the rebase table are unchanged

#### Scenario: Global shortcuts ignore focus
- **WHEN** the user presses Ctrl+S or Ctrl+H with either pane focused
- **THEN** the editor saves or toggles the shortcut reference, as it would with the left pane focused

### Requirement: Focus changes by mouse and keyboard
The user SHALL be able to move focus by clicking a pane, by pressing Alt+Left or Alt+Right, and by
pressing Esc while the right pane is focused. Because many macOS terminals deliver Alt+Left and
Alt+Right as the word-motion sequences `ESC b` and `ESC f`, those sequences SHALL be treated as the
same focus keys.

#### Scenario: Click to focus
- **WHEN** the user clicks inside the right pane
- **THEN** the right pane gains focus

#### Scenario: Keyboard focus right
- **WHEN** the left pane has focus and the user presses Alt+Right
- **THEN** the right pane gains focus

#### Scenario: Keyboard focus left
- **WHEN** the right pane has focus and the user presses Alt+Left
- **THEN** the left pane gains focus

#### Scenario: Word-motion sequence from a macOS terminal
- **WHEN** the terminal delivers `ESC f` while the left pane has focus
- **THEN** the right pane gains focus, exactly as for Alt+Right

#### Scenario: Esc leaves the right pane
- **WHEN** the right pane has focus and the user presses Esc
- **THEN** the left pane gains focus
- **AND** the editor does not exit

#### Scenario: Focus keys with no right pane
- **WHEN** the right pane is not drawn and the user presses Alt+Right
- **THEN** focus stays on the left pane and nothing else changes

### Requirement: Mouse wheel scrolls the pane under the pointer
Mouse wheel events SHALL scroll the pane beneath the pointer, whether or not it has focus. Mouse
capture SHALL be enabled for the whole session so wheel and click events reach the editor.

#### Scenario: Wheel over the unfocused status pane
- **WHEN** the left pane has focus and the user scrolls the wheel over the right pane
- **THEN** the right pane scrolls
- **AND** focus stays on the left pane

#### Scenario: Wheel over the rebase table
- **WHEN** the user scrolls the wheel over the rebase table
- **THEN** the selection moves one instruction per wheel step in the scroll direction

### Requirement: Narrow terminals show one pane at a time
When the terminal is narrower than 100 columns, the editor SHALL show only the left pane at full
width. Ctrl+T SHALL swap the visible pane to the right pane at full width and give it focus, and
pressing Ctrl+T or Esc there SHALL return to the left pane.

#### Scenario: Opening in a narrow terminal
- **WHEN** the editor opens in an 80-column terminal
- **THEN** only the left pane is shown, at full width

#### Scenario: Showing context in a narrow terminal
- **WHEN** only the left pane is shown and the user presses Ctrl+T
- **THEN** the right pane is shown at full width and has focus

#### Scenario: Returning to the editor
- **WHEN** the right pane is shown full width and the user presses Ctrl+T or Esc
- **THEN** the left pane is shown at full width and has focus
- **AND** the edits and cursor position are as they were

#### Scenario: Ctrl+T in a wide terminal
- **WHEN** both panes are side by side and the user presses Ctrl+T
- **THEN** focus moves to the other pane

### Requirement: Key bar reflects focus and mode
A single line below the panes SHALL list the most useful shortcuts for the focused pane in the current
mode, and SHALL always include save, cancel, and help.

#### Scenario: Editing a message
- **WHEN** the message pane has focus
- **THEN** the key bar lists save, cancel, help, and focus switching

#### Scenario: Browsing status
- **WHEN** the status pane has focus
- **THEN** the key bar lists the scrolling keys and how to return to the editor

#### Scenario: Planning a rebase
- **WHEN** the rebase table has focus
- **THEN** the key bar lists action keys, reordering, reword, and the raw text toggle

#### Scenario: Key bar wider than the terminal
- **WHEN** the listed shortcuts do not fit the terminal width
- **THEN** trailing entries are omitted rather than wrapped, and save, cancel, and help remain
