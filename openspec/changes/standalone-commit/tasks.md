## 1. Argument handling

- [ ] 1.1 Make the file path argument optional
- [ ] 1.2 Route a missing argument to standalone commit mode
- [ ] 1.3 Keep the existing behavior unchanged when a path is given
- [ ] 1.4 Report a usage error for more than one argument

## 2. Preflight checks

- [ ] 2.1 Detect whether the working directory is inside a git repository
- [ ] 2.2 Report and exit non-zero when it is not
- [ ] 2.3 Detect whether anything is staged
- [ ] 2.4 Report and exit non-zero when nothing is staged, before touching the terminal

## 3. Editing

- [ ] 3.1 Open the editor with an empty message area
- [ ] 3.2 Use the same message editor as a commit message file, with no right pane
- [ ] 3.3 Exit without committing on cancel

## 4. Committing

- [ ] 4.1 Write the message to a temporary file
- [ ] 4.2 Drop the terminal guard (mouse capture, alternate screen, raw mode) before invoking git
- [ ] 4.3 Invoke `git commit -F` with the temporary file
- [ ] 4.4 Show git's output to the user, including on failure
- [ ] 4.5 Exit with git's exit code
- [ ] 4.6 Remove the temporary file on every path

## 5. Verification

- [ ] 5.1 End-to-end check against a real repository: successful commit
- [ ] 5.2 End-to-end check: nothing staged
- [ ] 5.3 End-to-end check: empty message
- [ ] 5.4 End-to-end check: a commit hook that rejects the message
- [ ] 5.5 End-to-end check: run outside a repository
- [ ] 5.6 Confirm the terminal is usable and git's output readable after each of the above
