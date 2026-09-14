## 1. Subject wrapping

- [x] 1.1 Compute the available subject width from the terminal width minus the fixed columns
- [x] 1.2 Wrap a subject into lines of at most that width
- [x] 1.3 Prefer the last word boundary within the width
- [x] 1.4 Break mid-word when a single word exceeds the width
- [x] 1.5 Return a single line unchanged when the subject fits
- [x] 1.6 Handle a zero available width without panicking

## 2. Variable row heights

- [x] 2.1 Pre-compute the line count of each row from its wrapped subject
- [x] 2.2 Set each table row's height from that count
- [x] 2.3 Account for differing row heights when computing the scroll position

## 3. Verification

- [x] 3.1 Unit tests for wrapping at word boundaries
- [x] 3.2 Unit tests for the over-long word and zero-width edge cases
- [x] 3.3 Confirm the todo file written back is unaffected by display wrapping
