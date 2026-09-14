# git-context-detection Specification

## Purpose
Determines which git operation invoked the editor, so it can choose the right layout — message editor
with status pane, or rebase table with commit details — without the user having to say which one they
are in.

## Requirements

### Requirement: Operation detected from filename
The editor SHALL determine the git operation from the filename of the path it was given, recognizing
at minimum `COMMIT_EDITMSG`, `MERGE_MSG`, `git-rebase-todo`, `SQUASH_MSG`, and `TAG_EDITMSG`.

#### Scenario: Commit message file
- **WHEN** the editor is invoked on a file named `COMMIT_EDITMSG`
- **THEN** the detected operation is commit

#### Scenario: Merge message file
- **WHEN** the editor is invoked on a file named `MERGE_MSG`
- **THEN** the detected operation is merge

#### Scenario: Rebase todo file
- **WHEN** the editor is invoked on a file named `git-rebase-todo`
- **THEN** the detected operation is rebase

#### Scenario: Squash message file
- **WHEN** the editor is invoked on a file named `SQUASH_MSG`
- **THEN** the detected operation is squash

#### Scenario: Tag message file
- **WHEN** the editor is invoked on a file named `TAG_EDITMSG`
- **THEN** the detected operation is tag

#### Scenario: Unrecognized file
- **WHEN** the editor is invoked on a file whose name matches none of the known git files
- **THEN** the detected operation is unknown
- **AND** the file is still editable as plain text

#### Scenario: Only the filename matters
- **WHEN** the path contains directories, such as `/some/repo/.git/COMMIT_EDITMSG`
- **THEN** detection uses only the final path segment and the leading directories are ignored

### Requirement: Detected operation selects the layout
The detected operation SHALL determine the layout. Commit, merge, squash, and tag SHALL use the message
layout: the message editor with the status pane. Rebase SHALL use the rebase layout: the table with
commit details. Unknown SHALL use a plain text editor with no split and no right pane.

#### Scenario: Rebase gets the table
- **WHEN** the detected operation is rebase
- **THEN** the rebase table and commit details are presented

#### Scenario: Message operations share one layout
- **WHEN** the detected operation is commit, merge, squash, or tag
- **THEN** the message editor and status pane are presented, with no operation-specific restrictions

#### Scenario: Unknown file
- **WHEN** the detected operation is unknown
- **THEN** the whole file is shown in a plain text editor at full width

### Requirement: Git directory derived from the path
For a known git file, the editor SHALL treat the directory that holds it as the git directory, except
for `git-rebase-todo`, whose git directory is the parent of the `rebase-merge` directory that holds it.
This directory locates stored reword messages and rebase state.

#### Scenario: Commit message in a worktree
- **WHEN** the editor is invoked on `/repo/.git/worktrees/feature/COMMIT_EDITMSG`
- **THEN** the git directory is `/repo/.git/worktrees/feature`

#### Scenario: Rebase todo
- **WHEN** the editor is invoked on `/repo/.git/rebase-merge/git-rebase-todo`
- **THEN** the git directory is `/repo/.git`

### Requirement: Reword in progress detected
When the operation is commit and `<gitdir>/rebase-merge/done` exists, the editor SHALL read its last
line. If that line is a reword instruction whose commit reference matches a stored reword message
(one reference a prefix of the other), the editor SHALL treat the session as that reword, as specified
in `rebase-mode`.

#### Scenario: Matching stored message
- **WHEN** the last done line is `reword 45f8bcd subject` and `<gitdir>/gitmedit/reword/45f8bcd` exists
- **THEN** the session is detected as rewording `45f8bcd`

#### Scenario: No rebase in progress
- **WHEN** `<gitdir>/rebase-merge/done` does not exist
- **THEN** no reword is detected and the file opens as written

#### Scenario: Unreadable rebase state
- **WHEN** the done file cannot be read or parsed
- **THEN** no reword is detected and the editor starts normally

### Requirement: Configured comment character
The editor SHALL read git's configured comment character at startup and use it to recognize comment
lines, falling back to `#` whenever the configuration cannot be resolved.

#### Scenario: Custom comment character configured
- **WHEN** git is configured with a comment character other than `#`
- **THEN** lines beginning with that character are recognized as comments

#### Scenario: No configuration set
- **WHEN** git reports no configured comment character
- **THEN** `#` is used

#### Scenario: Automatic mode configured
- **WHEN** git is configured to choose the comment character automatically
- **THEN** `#` is used, since automatic selection is not supported

#### Scenario: git unavailable
- **WHEN** git cannot be executed to read the configuration
- **THEN** `#` is used and the editor starts normally
