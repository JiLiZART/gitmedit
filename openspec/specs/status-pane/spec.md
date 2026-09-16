# status-pane Specification

## Purpose
Turns the comment block git writes into a message file — branch state, conflicts, staged and unstaged
files, submodules, rebase progress — into a structured, colored, scrollable view in the right pane, so
the user can see what they are committing without reading raw comment text. The pane is display
only: nothing shown or done here changes what is saved.

## Requirements

### Requirement: Comment block parsed into sections
The status pane SHALL parse the comment trailer of a message file into sections, recognizing git's
headings regardless of how many files each lists. It SHALL recognize at minimum:

- **Branch**: `On branch`, `HEAD detached at`, ahead/behind/diverged/up-to-date lines, `Date:`, and
  `Author:`
- **Merge**: the "It looks like you may be committing a merge" notice and `All conflicts fixed but you
  are still merging`
- **Rebase**: `interactive rebase in progress; onto`, `Last command(s) done`, `Next command(s) to do`,
  and `You are currently rebasing`
- **Squash**: `This is a combination of N commits` and the per-commit headings that follow it
- **Conflicts**: `Conflicts:`
- **Staged**: `Changes to be committed:`
- **Unstaged**: `Changes not staged for commit:`
- **Submodules staged**: `Submodule changes to be committed:`
- **Submodules not updated**: `Submodules changed but not updated:`
- **Untracked**: `Untracked files:`
- **Diff**: everything below a scissors line

#### Scenario: Amend with staged files
- **WHEN** `fixtures/ammend_fixture.txt` is opened
- **THEN** the pane shows a Branch section with the branch name and the diverged state, and a Staged
  section listing all 34 files

#### Scenario: Merge with conflicts and many deletions
- **WHEN** `fixtures/merge_fixture2.txt` is opened
- **THEN** the pane shows Merge, Conflicts, Staged, Unstaged, Submodules staged, and Submodules not
  updated sections, each containing the entries git listed under that heading

#### Scenario: Rebase in progress
- **WHEN** `fixtures/rebase_fixture.txt` is opened
- **THEN** the pane shows a Rebase section with the onto commit, the commands done, and the commands
  remaining
- **AND** it shows the Untracked section listing `vendor/money-tree/`

#### Scenario: Conflicts only
- **WHEN** `fixtures/pull_rebase_fixture.txt` is opened
- **THEN** the pane shows a Conflicts section listing six files and no other sections

#### Scenario: Instruction boilerplate omitted
- **WHEN** the trailer contains git's "Please enter the commit message" instructions
- **THEN** those instruction lines are not shown in the pane

#### Scenario: Unrecognized comment lines kept visible
- **WHEN** the trailer contains comment lines that belong to no recognized section
- **THEN** they are shown verbatim in an "Other" section at the end, so no information is hidden

#### Scenario: Custom comment character
- **WHEN** git is configured with a comment character other than `#`
- **THEN** sections are parsed using that character

### Requirement: File entries shown with status badges
Each file entry SHALL be shown as a one-letter status badge followed by the path. The badges are
`M` modified, `A` new file, `D` deleted, `R` renamed, `C` copied, `T` typechange, and `U` for unmerged
states such as both modified. Badges SHALL be colored by meaning: additions green, deletions red,
modifications yellow, unmerged and conflict entries red and bold. Anything git adds after the path,
such as `(new commits)` or `(untracked content)`, SHALL be kept after the path, dimmed.

#### Scenario: Mixed staged changes
- **WHEN** the Staged section contains `modified:`, `new file:`, and `deleted:` entries
- **THEN** they are shown with `M`, `A`, and `D` badges in yellow, green, and red

#### Scenario: Renamed file
- **WHEN** an entry reads `renamed: old/path -> new/path`
- **THEN** it is shown with an `R` badge and both paths

#### Scenario: Submodule suffix
- **WHEN** an entry reads `modified: packages/@dev-kit (new commits)`
- **THEN** it is shown as `M packages/@dev-kit` followed by a dimmed `(new commits)`

#### Scenario: Conflict entries without a status word
- **WHEN** the Conflicts section lists bare paths
- **THEN** each is shown with a `U` badge

#### Scenario: Submodule summaries
- **WHEN** a submodule section contains `* path old...new (n):` lines, `>`/`<` commit lines, and
  `Warn:` lines
- **THEN** each submodule is shown as a heading with its commit lines indented beneath, and warnings
  are styled as warnings

### Requirement: Section headers show counts
Each section that lists files or commits SHALL be headed by its title and the number of entries it
contains.

#### Scenario: Count in header
- **WHEN** the Staged section lists 150 files
- **THEN** its header reads "Staged (150)"

#### Scenario: Empty section omitted
- **WHEN** git emitted a heading with no entries beneath it
- **THEN** the section is shown with a count of 0 rather than being dropped

### Requirement: Branch summary is compact
The Branch section SHALL present branch state in one or two lines instead of git's prose. It SHALL
show the branch name (or the detached commit), the number of commits ahead and behind the upstream
when present, and the date and author when present.

#### Scenario: Ahead of upstream
- **WHEN** the trailer reads `Your branch is ahead of 'origin/x' by 1 commit.`
- **THEN** the Branch section shows the branch name, the upstream name, and "ahead 1"

#### Scenario: Diverged
- **WHEN** the trailer reports that the branch and its upstream have diverged by 1 and 2 commits
- **THEN** the Branch section shows "ahead 1, behind 2"

### Requirement: Pane scrolls to the end of any list
The status pane SHALL scroll through its whole content however long it is. With the pane focused,
Up/Down SHALL scroll by one line, PgUp/PgDn by one page, and Home/End to the top and bottom. The mouse
wheel SHALL scroll it as described in `pane-layout`. A scroll position indicator SHALL be shown when
the content is taller than the pane.

#### Scenario: Reaching the last file of a long list
- **WHEN** `fixtures/merge_fixture2.txt` is open, the status pane is focused, and the user presses End
- **THEN** the last lines of the final section are visible

#### Scenario: Scrolling does not pass the ends
- **WHEN** the pane is scrolled to the top and the user presses Up
- **THEN** the view stays at the top

#### Scenario: Short content
- **WHEN** the whole content fits in the pane
- **THEN** no scroll indicator is shown and scroll keys change nothing

#### Scenario: Long paths
- **WHEN** a path is wider than the pane
- **THEN** it wraps onto continuation lines indented past the badge, rather than being truncated

### Requirement: Diff section for verbose commits
When the file contains a scissors line, as `git commit -v` writes, everything below it SHALL be shown
in a Diff section at the end of the pane. Added lines SHALL be shown in green, removed lines in red,
and hunk headers in a distinct color.

#### Scenario: Verbose commit
- **WHEN** a commit message file containing a scissors line followed by a diff is opened
- **THEN** the diff appears in the status pane's Diff section and not in the message editor

### Requirement: Status pane never changes what is saved
Parsing and displaying the status pane SHALL have no effect on the file written back. A failure to
parse any part of the trailer SHALL degrade to showing those lines in the Other section.

#### Scenario: Save after browsing
- **WHEN** the user scrolls the status pane and then saves
- **THEN** the trailer is written back exactly as it was read

#### Scenario: Malformed block
- **WHEN** a recognized heading is followed by lines in an unexpected format
- **THEN** the editor opens normally and those lines are shown verbatim
