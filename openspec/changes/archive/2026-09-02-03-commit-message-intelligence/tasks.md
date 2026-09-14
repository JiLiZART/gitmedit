## 1. Subject line feedback

- [x] 1.1 Expose the first editable line as the subject
- [x] 1.2 Render its character count in the status bar
- [x] 1.3 Color the count by the 50 and 72 character thresholds
- [x] 1.4 Handle the no-editable-content case without erroring

## 2. Blank separator detection

- [x] 2.1 Detect whether the second editable line is blank
- [x] 2.2 Treat messages shorter than two lines as satisfying the rule
- [x] 2.3 Surface the missing separator in the status bar

## 3. Help overlay

- [x] 3.1 Add overlay visibility to the editor state, toggled by Ctrl+H
- [x] 3.2 Render the overlay centered over the current view
- [x] 3.3 Build the shortcut list from the detected git operation
- [x] 3.4 Include the read-only restrictions that apply per operation

## 4. Input isolation

- [x] 4.1 Swallow all input except the dismiss keys while the overlay is visible
- [x] 4.2 Dismiss on Esc without exiting the editor
- [x] 4.3 Dismiss on Ctrl+H

## 5. Verification

- [x] 5.1 Unit tests for each color threshold boundary
- [x] 5.2 Unit tests for blank separator detection including the short-message case
- [x] 5.3 Unit tests for overlay visibility transitions
