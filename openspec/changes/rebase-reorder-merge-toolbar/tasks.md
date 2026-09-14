## 1. Reordering — test first

- [ ] 1.1 Write the failing test: `pick A`, comment, `pick B`, move B up, assert A and B exchanged and the comment did not move
- [ ] 1.2 Write the failing test: moving the first instruction up leaves the todo unchanged
- [ ] 1.3 Write the failing test: no instruction lost or duplicated after a sequence of moves

## 2. Reordering — implementation

- [ ] 2.1 Find the previous and next instruction positions, skipping non-instruction lines
- [ ] 2.2 Exchange the two instructions, leaving every other line in place
- [ ] 2.3 Bind Alt+Up and Alt+Down to the moves
- [ ] 2.4 Regenerate the selectable index from scratch after each move
- [ ] 2.5 Re-derive the selection from the moved instruction's new position

## 3. Exec command editing

- [ ] 3.1 Add an editing state to the rebase view, entered with Enter on an exec instruction
- [ ] 3.2 Route all key input to the editing state while it is active
- [ ] 3.3 Commit the edit on Ctrl+S and return to navigation
- [ ] 3.4 Discard the edit on Esc without exiting the editor
- [ ] 3.5 Ignore Enter on non-exec instructions
- [ ] 3.6 Serialize the edited command in git's exec instruction format

## 4. Merge conflict display

- [ ] 4.1 Parse the conflicted-file list from the merge message comment block
- [ ] 4.2 Display it in merge mode
- [ ] 4.3 Display nothing when no block is present or the block is not recognized
- [ ] 4.4 Degrade without overflow when the list is longer than the space available
- [ ] 4.5 Confirm the comment block is still written back byte for byte

## 5. Verification

- [ ] 5.1 Round-trip tests for reordered todos containing comments and unrecognized lines
- [ ] 5.2 Navigation tests after reordering
- [ ] 5.3 Row height and alignment tests with wrapped subjects after a move
- [ ] 5.4 Tests that navigation keys do nothing while an exec command is being edited
