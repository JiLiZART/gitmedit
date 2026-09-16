## Purpose

Lets the editor be the entry point for making a commit: running it with no arguments starts
`git commit` with gitmedit as the message editor, and reports exactly what git reports.

## ADDED Requirements

### Requirement: No arguments starts a commit in gitmedit
When invoked with no arguments, the editor SHALL run `git commit` in the current working directory
with gitmedit set as git's editor for that invocation only, overriding `core.editor` without changing
any git configuration. The editor SHALL NOT set up the terminal itself; the gitmedit instance that git
launches does.

#### Scenario: Staged changes
- **WHEN** the user runs `gitmedit` with no arguments in a repository with staged changes
- **THEN** `git commit` starts and the commit message opens in gitmedit's message layout
- **AND** the status pane lists the staged files

#### Scenario: Another editor configured
- **WHEN** `core.editor` is set to a different editor and the user runs `gitmedit` with no arguments
- **THEN** the commit message still opens in gitmedit
- **AND** `core.editor` is unchanged afterwards

#### Scenario: Installed in a path containing spaces
- **WHEN** the gitmedit binary lives in a directory whose path contains spaces or quotes
- **THEN** git still launches it as the editor

### Requirement: Outcome and exit code come from git
The editor SHALL leave git's own output visible, let git decide the outcome, and exit with git's exit
code. The editor SHALL NOT pre-check the repository or the staged changes itself.

#### Scenario: Commit saved
- **WHEN** the user writes a message and saves
- **THEN** the commit is created, git's commit summary is shown, and the process exits with code 0

#### Scenario: Commit cancelled
- **WHEN** the user cancels the message editor
- **THEN** no commit is created, git's abort message is shown, and the process exits with git's
  non-zero code

#### Scenario: Nothing to commit
- **WHEN** the user runs `gitmedit` with no arguments and nothing is staged
- **THEN** git's "nothing to commit" report is shown, no editor opens, and the process exits with git's
  non-zero code

#### Scenario: Outside a repository
- **WHEN** the user runs `gitmedit` with no arguments outside any git repository
- **THEN** git's "not a git repository" error is shown, no editor opens, and the process exits with
  git's non-zero code

#### Scenario: A hook rejects the commit
- **WHEN** a commit hook rejects the commit
- **THEN** the hook's output is shown and the process exits with git's code

#### Scenario: git cannot be run
- **WHEN** git is not installed or cannot be executed
- **THEN** an error saying git could not be run is written to stderr and the process exits with a
  non-zero code
