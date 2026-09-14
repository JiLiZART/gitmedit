## Purpose

Determines which git operation invoked the editor, so the interface can adapt to commit, merge,
rebase, squash, and tag work without the user having to say which one they are in.

## ADDED Requirements

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
- **THEN** the detected operation is treated as commit, since a tag message is edited the same way

#### Scenario: Unrecognized file
- **WHEN** the editor is invoked on a file whose name matches none of the known git files
- **THEN** the detected operation is unknown
- **AND** the file is still editable as plain text

#### Scenario: Only the filename matters
- **WHEN** the path contains directories, such as `/some/repo/.git/COMMIT_EDITMSG`
- **THEN** detection uses only the final path segment and the leading directories are ignored

### Requirement: Detected operation selects the interface
The detected operation SHALL determine which editing interface is presented, so that structured
operations get a structured interface rather than free-form text.

#### Scenario: Rebase gets a structured interface
- **WHEN** the detected operation is rebase
- **THEN** a structured, non-free-text interface is presented

#### Scenario: Message operations get a text interface
- **WHEN** the detected operation is commit, merge, or unknown
- **THEN** a free-form text editing interface is presented
