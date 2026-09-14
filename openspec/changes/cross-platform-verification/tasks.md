## 1. macOS

- [ ] 1.1 Verify commit mode in Terminal.app and iTerm2
- [ ] 1.2 Verify merge mode in both
- [ ] 1.3 Verify rebase mode in both, including wrapped subjects and row alignment
- [ ] 1.4 Verify squash mode in both
- [ ] 1.5 Verify standalone commit mode in both
- [ ] 1.6 Verify redraw after a terminal resize

## 2. Windows

- [ ] 2.1 Verify each mode renders correctly in Windows Terminal
- [ ] 2.2 Verify colors and box-drawing render as intended
- [ ] 2.3 Verify cursor placement matches the edited position

## 3. Key event handling

- [ ] 3.1 Confirm the press-event filter is sufficient on a real Windows console
- [ ] 3.2 Verify Tab cycles exactly one step per press
- [ ] 3.3 Verify Esc dismisses the shortcut reference without exiting
- [ ] 3.4 Verify a typed character is inserted exactly once
- [ ] 3.5 Verify the Alt-modified reordering keys are delivered as expected, or record a fallback

## 4. Follow-up

- [ ] 4.1 Record every artifact or mismatch found, with the terminal and version it appeared in
- [ ] 4.2 Raise a separate change for any fix the verification turns up
