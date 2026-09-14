## 1. Editing mode

- [x] 1.1 Define the editing mode with plain, merge, and squash variants
- [x] 1.2 Take the mode as a parameter when parsing a document
- [x] 1.3 Decide which line kinds are editable per mode when building the editable index
- [x] 1.4 Expose the mode on the parsed document

## 2. Serialization

- [x] 2.1 Short-circuit plain mode to emit the editing surface's lines directly
- [x] 2.2 Keep comment re-injection for merge and squash
- [x] 2.3 Take conflict markers from the editing surface in merge mode
- [x] 2.4 Keep conflict markers verbatim in squash mode

## 3. Wiring

- [x] 3.1 Map the detected git operation to an editing mode when constructing the editor state
- [x] 3.2 Pass the selected mode into document parsing
- [x] 3.3 Default unrecognized operations to plain mode

## 4. Status bar cleanup

- [x] 4.1 Remove the subject counter helper and its color thresholds
- [x] 4.2 Remove the blank-line warning helper
- [x] 4.3 Reduce the commit and squash status bars to the available keys
- [x] 4.4 Delete the tests covering the removed helpers

## 5. Verification

- [x] 5.1 Tests that comment lines are editable in commit mode
- [x] 5.2 Tests that comments stay protected in merge mode
- [x] 5.3 Tests that conflict markers are editable in merge mode
- [x] 5.4 Confirm existing squash and merge round-trip tests still pass
- [x] 5.5 Confirm the removed helpers are absent from the renderer
