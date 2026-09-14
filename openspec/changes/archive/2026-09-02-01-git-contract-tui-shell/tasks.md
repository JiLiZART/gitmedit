## 1. Process contract

- [x] 1.1 Parse the file path from the first command-line argument
- [x] 1.2 Exit with code 1 and a stderr message when the path does not exist
- [x] 1.3 Read the file contents into memory before entering the TUI
- [x] 1.4 Exit with code 0 after a successful save, and code 1 on cancel

## 2. Terminal lifecycle

- [x] 2.1 Install a panic hook that disables raw mode before printing, ahead of any terminal mutation
- [x] 2.2 Add an RAII guard that enables raw mode on construction and restores it on drop
- [x] 2.3 Construct the crossterm backend manually so the alternate screen is never entered
- [x] 2.4 Drop the terminal guard before writing the file on save

## 3. Atomic write-back

- [x] 3.1 Write content to a temporary file beside the target
- [x] 3.2 Rename the temporary file over the target
- [x] 3.3 Verify no temporary file survives a successful write
- [x] 3.4 Verify unix line endings are preserved through a roundtrip

## 4. Context detection

- [x] 4.1 Map filenames to git operations: COMMIT_EDITMSG, MERGE_MSG, git-rebase-todo, SQUASH_MSG, TAG_EDITMSG
- [x] 4.2 Use only the final path segment, ignoring leading directories
- [x] 4.3 Fall back to an unknown operation for unrecognized filenames
- [x] 4.4 Select the structured interface for rebase and the text interface for everything else

## 5. Verification

- [x] 5.1 Unit tests for each filename mapping
- [x] 5.2 Unit tests for atomic write behavior
- [x] 5.3 PTY integration test asserting no alternate screen escape sequences are emitted
- [x] 5.4 Measure startup latency against the sub-100ms budget
