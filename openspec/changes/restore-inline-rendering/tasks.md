## 1. Establish intent

- [ ] 1.1 Determine whether commit `8dd89f1` was a deliberate reversal or a merge accident
- [ ] 1.2 If deliberate, stop and revise the specs and README instead of implementing this change

## 2. Inline rendering

- [ ] 2.1 Remove the alternate-screen switch from terminal guard construction
- [ ] 2.2 Remove mouse capture, which no event handler consumes
- [ ] 2.3 Remove the corresponding leave and disable calls from the cleanup path

## 3. Panic-safe teardown

- [ ] 3.1 Suppress errors in the guard's cleanup path instead of unwrapping them
- [ ] 3.2 Align the panic hook with whatever state the guard actually switches on
- [ ] 3.3 Verify cleanup during an unwind does not abort the process

## 4. Regression coverage

- [ ] 4.1 Add a test asserting no alternate-screen escape sequences are emitted, running without a PTY
- [ ] 4.2 Decide whether to un-ignore or retire the existing PTY test
- [ ] 4.3 Confirm the new test fails when the alternate screen is reintroduced

## 5. Verification

- [ ] 5.1 Run the full suite and confirm it passes with no ignored tests masking the guarantee
- [ ] 5.2 Manually confirm prior terminal output remains visible on entry and in scrollback on exit
- [ ] 5.3 Manually confirm the terminal is usable after a forced panic
