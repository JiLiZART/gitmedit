## 1. Layout

- [ ] 1.1 Define the three-region vertical split once at the top level
- [ ] 1.2 Pass the middle region to every mode's content renderer
- [ ] 1.3 Confirm no sub-renderer reads the full frame

## 2. Header bar

- [ ] 2.1 Derive the project folder and file name from the path the editor was given
- [ ] 2.2 Render the folder on the left and the file name on the right
- [ ] 2.3 Degrade gracefully when the terminal is too narrow for both

## 3. Command bar

- [ ] 3.1 Define the shortcut set per git operation in one place
- [ ] 3.2 Render the applicable set at the bottom of every mode
- [ ] 3.3 Show rebase navigation keys in rebase mode and editing keys in message modes

## 4. Shortcut reference correction

- [ ] 4.1 Build the reference from the same per-operation definition as the command bar
- [ ] 4.2 Report comment lines as read-only in merge mode, and conflict markers as editable
- [ ] 4.3 Report no restrictions in commit mode

## 5. Verification

- [ ] 5.1 Test cursor placement at the bottom of a message longer than the content area
- [ ] 5.2 Test scrolling against the reduced content height
- [ ] 5.3 Test rebase table alignment with the bars present
- [ ] 5.4 Test that the command bar and shortcut reference agree for every operation
