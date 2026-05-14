# Roadmap: gitmedit

## Milestones

- ✅ **v1.0 MVP** — Phases 1-6 (shipped 2026-04-07)
- 🚧 **v1.1 Standalone Commit + Editor Overhaul** — Phases 7-12 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-6) — SHIPPED 2026-04-07</summary>

- [x] Phase 1: Git Contract + TUI Shell (3/3 plans) — completed 2026-03-18
- [x] Phase 2: Text Editing + Comment Handling (3/3 plans) — completed 2026-03-21
- [x] Phase 3: Commit Message Intelligence (2/2 plans) — completed 2026-03-22
- [x] Phase 4: Rebase + Squash Modes (3/3 plans) — completed 2026-03-23
- [x] Phase 5: Installation + Distribution (2/2 plans) — completed 2026-03-27
- [x] Phase 6: Rebase View Horizontal Scrolling (1/1 plans) — completed 2026-04-02

**Known gaps (from audit):** IO-06 regression (alternate screen), Phase 2 EDIT-03/EDIT-07/PERF-02/PERF-03 pending, Phase 5 INSTALL-05 pending. See milestones/v1.0-MILESTONE-AUDIT.md.

</details>

### 🚧 v1.1 Standalone Commit + Editor Overhaul (In Progress)

**Milestone Goal:** Add standalone commit mode, simplify editor to nano-like behavior with nano-style chrome, add merge toolbar intelligence, and close v1.0 regression gaps.

- [x] **Phase 7: Foundations Fix** - Fix IO-06 alternate screen regression and TerminalGuard Drop panic (completed 2026-04-22)
- [ ] **Phase 8: Plain Editor Default** - Remove comment line read-only protection; use textarea default behavior
- [ ] **Phase 9: Nano Chrome** - Add header bar (folder + filename) and bottom command bar across all modes
- [ ] **Phase 10: Rebase Enhancements + Merge Toolbar** - Rebase line reordering, exec editing, merge conflict toolbar
- [ ] **Phase 11: Standalone Commit Mode** - `gitmedit` with no args opens commit editor and invokes `git commit -F`
- [ ] **Phase 12: Cross-Platform Verification** - Verify correct rendering and key handling on macOS and Windows
- [ ] **Phase 13: Colorize COMMIT_MSG Comment Sections** - Context-aware colorization of `#` comment sections (instructions, branch info, conflicts, staged changes, rebase status)

## Phase Details

### Phase 7: Foundations Fix
**Goal**: Terminal renders correctly without alternate screen and guard cleanup never panics
**Depends on**: Nothing (first v1.1 phase)
**Requirements**: IO-06, IO-07
**Success Criteria** (what must be TRUE):
  1. Running `gitmedit` no longer switches to an alternate screen; output appears inline in the terminal scroll buffer like nano
  2. Closing the editor (save or cancel) restores the terminal without any panic, even if a subsequent subprocess fails
  3. All 94 existing tests pass after the fix with zero regressions
**Plans**: TBD

### Phase 8: Plain Editor Default
**Goal**: All lines in commit, merge, and squash modes are editable by default with no read-only comment protection
**Depends on**: Phase 7
**Requirements**: EDIT-10, EDIT-11
**Success Criteria** (what must be TRUE):
  1. In commit mode, the user can position the cursor on a comment line (`# ...`) and type to overwrite it
  2. ratatui_textarea default editing behavior (cursor movement, insert, delete) works on every line without special-cased protection
  3. Squash and merge modes remain fully functional — their existing behavior is unchanged
  4. Existing squash and merge roundtrip tests pass after the Document internals change
**Plans:** 3 plans
Plans:
- [ ] 08-01-PLAN.md — Add `EditorMode` enum and mode-aware `Document::parse`/`editable_lines`/`serialize` (with new + updated tests)
- [ ] 08-02-PLAN.md — Remove subject-line counter and blank-line warning from commit and squash status bars (delete `counter_color_for` / `blank_warning_span` helpers + their tests)
- [ ] 08-03-PLAN.md — Wire `EditorMode` into `App::new` via `GitContext` match and update `Document::parse` call; add commit/merge-mode behavior tests

### Phase 9: Nano Chrome
**Goal**: All editor modes display a top header bar and a bottom command bar in the nano style
**Depends on**: Phase 8
**Requirements**: UI-01, UI-02, UI-03, UI-04
**Success Criteria** (what must be TRUE):
  1. A header bar is visible at the top of every mode showing the project folder name on the left and the current filename on the right
  2. A command bar is visible at the bottom showing available shortcuts (e.g., `^S Save  Esc Cancel`)
  3. The command bar labels change to reflect the current mode — rebase mode shows rebase-specific hints, commit mode shows commit hints
  4. The content area (text editor or rebase table) still fills the remaining space correctly — scroll and cursor calculations are unaffected
**Plans**: TBD
**UI hint**: yes

### Phase 10: Rebase Enhancements + Merge Toolbar
**Goal**: Users can reorder and edit rebase lines, and merge mode shows conflict file information in the toolbar
**Depends on**: Phase 9
**Requirements**: UI-05, UI-06, REBASE-10, REBASE-11, REBASE-12, REBASE-13
**Success Criteria** (what must be TRUE):
  1. In rebase mode, pressing Alt+Up moves the selected line one position up; Alt+Down moves it one position down; the cursor follows the moved line
  2. After reordering, saving produces a correctly ordered rebase todo file — comment lines stay in place and no commits are lost or duplicated
  3. Selecting an exec line and pressing Enter opens an inline editor for the exec command argument; Ctrl+S commits the edit, Esc cancels
  4. In merge mode, the toolbar area displays the list of conflicted files parsed from the MERGE_MSG comment block
  5. If MERGE_MSG contains no conflict block, the toolbar shows nothing rather than empty or broken output
**Plans**: TBD
**UI hint**: yes

### Phase 11: Standalone Commit Mode
**Goal**: Users can run `gitmedit` with no arguments inside a git repo to write and submit a commit message
**Depends on**: Phase 7
**Requirements**: COMMIT-10, COMMIT-11, COMMIT-12, COMMIT-13, COMMIT-14
**Success Criteria** (what must be TRUE):
  1. Running `gitmedit` (no args) inside a git repo with staged changes opens the TUI with an empty commit message editor
  2. Pressing Ctrl+S writes the message and invokes `git commit -F <tmpfile>`, then exits with git's exit code
  3. Running `gitmedit` with no staged changes exits immediately with a clear error message before opening the TUI
  4. If `git commit` fails (e.g., a commit hook rejects the message), the error output from git is shown to the user before the editor exits
**Plans**: TBD

### Phase 12: Cross-Platform Verification
**Goal**: The editor renders and responds to input correctly on all supported terminal environments
**Depends on**: Phases 7-11 (all features must be stable)
**Requirements**: PLAT-01, PLAT-02, PLAT-03
**Success Criteria** (what must be TRUE):
  1. The editor renders without visual artifacts on macOS Terminal.app and iTerm2 across all modes (commit, merge, rebase, squash, standalone)
  2. The editor renders without visual artifacts on Windows Terminal (ConPTY)
  3. Key events on Windows do not fire twice — each keypress produces exactly one action (no double-fire from ConPTY press/release duplication)
**Plans**: TBD

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Git Contract + TUI Shell | v1.0 | 3/3 | Complete | 2026-03-18 |
| 2. Text Editing + Comment Handling | v1.0 | 3/3 | Complete | 2026-03-21 |
| 3. Commit Message Intelligence | v1.0 | 2/2 | Complete | 2026-03-22 |
| 4. Rebase + Squash Modes | v1.0 | 3/3 | Complete | 2026-03-23 |
| 5. Installation + Distribution | v1.0 | 2/2 | Complete | 2026-03-27 |
| 6. Rebase View Horizontal Scrolling | v1.0 | 1/1 | Complete | 2026-04-02 |
| 7. Foundations Fix | v1.1 | 1/1 | Complete   | 2026-04-22 |
| 8. Plain Editor Default | v1.1 | 0/3 | Not started | - |
| 9. Nano Chrome | v1.1 | 0/? | Not started | - |
| 10. Rebase Enhancements + Merge Toolbar | v1.1 | 0/? | Not started | - |
| 11. Standalone Commit Mode | v1.1 | 0/? | Not started | - |
| 12. Cross-Platform Verification | v1.1 | 0/? | Not started | - |
| 13. Colorize COMMIT_MSG Comment Sections | v1.1 | 0/? | Not started | - |

### Phase 13: Colorize COMMIT_MSG Comment Sections

**Goal**: Comment lines in COMMIT_MSG files are visually distinct and context-aware — each semantic section (instructions, branch info, conflicts, staged changes, rebase status) renders in a different color so users can scan at a glance without reading every `#` line
**Depends on**: Phase 8 (Plain Editor Default — comment lines must be editable before colorizing them)
**Requirements**: UI-10, UI-11, UI-12
**Success Criteria** (what must be TRUE):
  1. Instructions header (`# Please enter the commit message…`) renders dimmed/muted — visually de-emphasized
  2. Branch/status lines (`# On branch`, `# Your branch is…`, `# Author:`, `# Date:`) render in a neutral info color (e.g. cyan/blue)
  3. Conflict file list (`# Conflicts:` + file entries) renders in a warning color (e.g. yellow/amber)
  4. Staged changes section (`# Changes to be committed:`, `#   modified:`, `#   new file:`, `#   deleted:`) renders in green for new/modified, red for deleted
  5. Rebase status block (`# interactive rebase in progress`, `# Last commands done`, `# pick …`, `# Next command to do`) renders in a distinct rebase color (e.g. magenta/purple)
  6. Submodule info lines render in the same neutral info color as branch lines
  7. All 94 existing tests pass; colorization is rendering-only and does not affect saved file content
**Comment section taxonomy** (derived from fixtures: ammend, merge, rebase, pull_rebase):
  - `instructions` — `# Please enter…` / `# Lines starting with '#'…`
  - `branch-info` — `# On branch`, `# Your branch`, `# Author:`, `# Date:`
  - `merge-notice` — `# It looks like you may be committing a merge`
  - `rebase-status` — `# interactive rebase in progress`, `# Last commands done`, `# Next command to do`, `# You are currently rebasing`
  - `conflicts` — `# Conflicts:` header + file list entries
  - `changes-staged` — `# Changes to be committed:` + file list
  - `changes-unstaged` — `# Changes not staged for commit:` + file list
  - `submodule-info` — `# Submodule changes to be committed:`, `# Submodules changed but not updated:`
  - `rebase-pick` — `#   pick <hash> # <msg>` lines
**Plans**: TBD

| 13. Colorize COMMIT_MSG Comment Sections | v1.1 | 0/? | Not started | - |
