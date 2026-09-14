## Purpose

Classifies the comment block git generates into its semantic sections and colors each one by meaning,
so the user can find the branch, the staged files, or a conflict list without reading every line.

## ADDED Requirements

### Requirement: Comment lines are classified by section
Each comment line SHALL be classified into the section of git's generated block it belongs to,
carrying its classification from the section heading through the lines that follow it.

#### Scenario: Instruction boilerplate
- **WHEN** a line is part of git's "please enter the commit message" preamble
- **THEN** it is classified as instructions

#### Scenario: Branch and author information
- **WHEN** a line states the current branch, the branch's tracking status, an author, or a date
- **THEN** it is classified as branch information

#### Scenario: Merge notice
- **WHEN** a line notes that the commit may be a merge
- **THEN** it is classified as a merge notice

#### Scenario: Rebase status
- **WHEN** a line reports an interactive rebase in progress, the commands done, or the next command
- **THEN** it is classified as rebase status

#### Scenario: Rebase instruction list
- **WHEN** a line inside the rebase status block lists an individual instruction and its commit
- **THEN** it is classified as a rebase instruction

#### Scenario: Conflicts
- **WHEN** a line is the conflicts heading, or a file listed under it
- **THEN** it is classified as a conflict

#### Scenario: Staged and unstaged changes
- **WHEN** a line is the changes-to-be-committed or changes-not-staged heading, or a file listed
  under one of them
- **THEN** it is classified as that section, and the file entry's kind — modified, new, or deleted —
  is recorded

#### Scenario: Submodule information
- **WHEN** a line reports submodule changes
- **THEN** it is classified as submodule information

#### Scenario: Unrecognized comment
- **WHEN** a comment line matches no known section
- **THEN** it is classified as unrecognized and rendered in the default comment style

### Requirement: Sections are colored by meaning
Each classified section SHALL be rendered in a style that reflects what it means, so sections are
distinguishable at a glance.

#### Scenario: Instructions are de-emphasized
- **WHEN** instruction lines are rendered
- **THEN** they appear dimmed relative to the rest of the block

#### Scenario: Informational sections are neutral
- **WHEN** branch information or submodule information is rendered
- **THEN** both appear in the same neutral informational style

#### Scenario: Conflicts warn
- **WHEN** the conflicts section is rendered
- **THEN** it appears in a warning style, distinct from informational lines

#### Scenario: File changes signal their kind
- **WHEN** staged file entries are rendered
- **THEN** modified and new entries appear in a positive style and deleted entries in a negative one

#### Scenario: Rebase status is distinct
- **WHEN** the rebase status block is rendered
- **THEN** it appears in a style distinct from instructions, informational lines, and conflicts

### Requirement: Colorization does not affect saved content
Classification and coloring SHALL affect rendering only, and SHALL NOT change what is written back.

#### Scenario: Saving a colorized message
- **WHEN** a message whose comments were classified and colored is saved without editing
- **THEN** the bytes written are identical to the bytes read

#### Scenario: Editing a colorized comment
- **WHEN** the user edits a comment line and saves
- **THEN** the edited text is written back exactly as typed, with no styling artifacts

#### Scenario: Classification never fails the editor
- **WHEN** a comment block has an unexpected shape or wording
- **THEN** the affected lines fall back to the default comment style and the editor works normally
