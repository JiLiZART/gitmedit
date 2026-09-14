## 1. Comment character

- [x] 1.1 Read git's configured comment character at startup
- [x] 1.2 Fall back to `#` for unset, automatic, empty, and git-unavailable cases

## 2. Line classification

- [x] 2.1 Classify each line as content, comment, or conflict marker
- [x] 2.2 Detect both the six and seven character conflict-marker forms
- [x] 2.3 Build an index mapping editable rows to their position in the full line list

## 3. Editing surface

- [x] 3.1 Feed only editable lines into the text area
- [x] 3.2 Delegate unhandled keys to the text area for insertion, deletion, and cursor movement
- [x] 3.3 Bind Ctrl+U to delete the current line
- [x] 3.4 Bind Ctrl+W and Ctrl+D to delete the previous and next word
- [x] 3.5 Bind Ctrl+Z and Ctrl+Y to undo and redo
- [x] 3.6 Bind Ctrl+C, Ctrl+X, and Ctrl+V to the system clipboard, ignoring failures silently

## 4. Rendering

- [x] 4.1 Style comment lines distinctly from content
- [x] 4.2 Style conflict markers distinctly from both
- [x] 4.3 Scroll horizontally to keep the cursor visible on long lines
- [x] 4.4 Scroll vertically to keep the cursor line visible

## 5. Serialization

- [x] 5.1 Re-inject protected lines verbatim in their original positions on save
- [x] 5.2 Verify an unedited file round-trips byte for byte
- [x] 5.3 Verify comments survive edits to surrounding content
- [x] 5.4 Verify conflict markers survive edits to surrounding content
- [x] 5.5 Cover the empty-file and comment-only-file edge cases
