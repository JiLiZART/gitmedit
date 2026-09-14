## 1. macOS

- [ ] 1.1 Verify the message layout (commit, amend, merge, squash) in Terminal.app and iTerm2
- [ ] 1.2 Verify the rebase layout in both, including wrapped subjects, row alignment, and commit details
- [ ] 1.3 Verify standalone commit mode in both
- [ ] 1.4 Verify redraw after a terminal resize, including crossing the narrow-terminal threshold
- [ ] 1.5 Verify the previous terminal content is back after exit

## 2. Windows

- [ ] 2.1 Verify both layouts render correctly in Windows Terminal
- [ ] 2.2 Verify colors, badges, and box-drawing borders render as intended
- [ ] 2.3 Verify cursor placement matches the edited position, including in soft-wrapped lines

## 3. Key and mouse event handling

- [ ] 3.1 Confirm the press-event filter is sufficient on a real Windows console
- [ ] 3.2 Verify Tab cycles exactly one step per press
- [ ] 3.3 Verify Esc dismisses the shortcut reference without exiting
- [ ] 3.4 Verify a typed character is inserted exactly once
- [ ] 3.5 Verify Alt+Left/Right switch focus, including the `ESC b`/`ESC f` form macOS terminals send
- [ ] 3.6 Verify Alt+Up/Down reorder rebase instructions, or record a fallback
- [ ] 3.7 Verify click-to-focus and wheel scrolling over each pane

## 4. Follow-up

- [ ] 4.1 Record every artifact or mismatch found, with the terminal and version it appeared in
- [ ] 4.2 Raise a separate change for any fix the verification turns up
