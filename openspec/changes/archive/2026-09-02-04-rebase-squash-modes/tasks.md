## 1. Rebase todo parsing

- [x] 1.1 Define the typed line representation for instructions, comments, and other lines
- [x] 1.2 Parse action words in both long and abbreviated form
- [x] 1.3 Capture commit reference and full subject, including subjects containing spaces
- [x] 1.4 Preserve unrecognized lines verbatim
- [x] 1.5 Handle the empty todo file

## 2. Rebase todo serialization

- [x] 2.1 Emit instructions in git's format
- [x] 2.2 Preserve comment lines and line order
- [x] 2.3 Round-trip tests for unchanged files
- [x] 2.4 Round-trip tests for abbreviated action forms

## 3. Rebase interface

- [x] 3.1 Render instructions as a table with action, commit, and subject columns
- [x] 3.2 Color rows by action
- [x] 3.3 Build the selectable-row index excluding comments
- [x] 3.4 Bind arrow keys to move the selection, clamped at both ends
- [x] 3.5 Bind Tab to cycle the selected action
- [x] 3.6 Leave exec instructions unchanged when cycling
- [x] 3.7 Route rebase mode away from free-text key handling

## 4. Squash mode

- [x] 4.1 Detect the generated log header and its extent
- [x] 4.2 Split the raw content into header and message before parsing the document
- [x] 4.3 Fall back to treating the whole file as editable when no header is found
- [x] 4.4 Render the log in its own pane, labelled read-only
- [x] 4.5 Render the editable message below it
- [x] 4.6 Reassemble header and edited message on save

## 5. Verification

- [x] 5.1 Unit tests for each action parse form and the cycle order
- [x] 5.2 Round-trip tests for todo files with interleaved comments
- [x] 5.3 Tests for header detection including the no-header fallback
- [x] 5.4 Tests confirming header content is unchanged after editing the message
