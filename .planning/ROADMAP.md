# Roadmap: gitmedit

## Overview

gitmedit ships as a single Rust binary that git invokes as a blocking subprocess. The delivery path follows the git editor contract: correctness first (Phase 1 establishes the terminal/exit-code contract), then a working text editor for commits and merges (Phase 2), then commit-specific intelligence (Phase 3), then structured rebase and squash modes (Phase 4), and finally installation and distribution (Phase 5). Each phase delivers a coherent, verifiable capability before the next begins.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Git Contract + TUI Shell** - Binary opens a file, renders it in TUI, saves or cancels with correct exit codes, and never corrupts the terminal (completed 2026-03-18)
- [ ] **Phase 2: Text Editing + Comment Handling** - Full text editing for COMMIT_EDITMSG and MERGE_MSG with comment preservation, cursor movement, multiline, and merge conflict styling
- [x] **Phase 3: Commit Message Intelligence** - Subject line counter with 50/72 color coding, blank line enforcement, and context-aware hotkey help overlay (completed 2026-03-22)
- [x] **Phase 4: Rebase + Squash Modes** - Structured rebase-todo display with action cycling, squash context rendering with protected commit log (completed 2026-03-23)
- [ ] **Phase 5: Installation + Distribution** - cargo install path, PATH availability, and dual git config setup (core.editor + sequence.editor)

## Phase Details

### Phase 1: Git Contract + TUI Shell
**Goal**: The binary reads a file from argv[1], displays it in a TUI that does not use alternate screen, and exits with code 0 on save or code 1 on cancel — with terminal always restored even if the process panics
**Depends on**: Nothing (first phase)
**Requirements**: IO-01, IO-02, IO-03, IO-04, IO-05, IO-06, CTX-01, CTX-02, PERF-01
**Success Criteria** (what must be TRUE):
  1. Running `gitmedit /path/to/COMMIT_EDITMSG` opens a TUI showing the file contents without entering alternate screen (prior terminal output remains visible)
  2. Saving writes the file and returns exit code 0; git proceeds with the operation
  3. Cancelling (Esc) exits with code 1; git aborts the operation
  4. If the process panics mid-session, the terminal is restored to normal mode (no stuck raw mode)
  5. The binary starts in under 100ms on typical hardware
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md — Project scaffold, context detection, terminal safety
- [x] 01-02-PLAN.md — TUI rendering with ratatui, status bar
- [x] 01-03-PLAN.md — Event loop, file writer, exit code wiring

### Phase 2: Text Editing + Comment Handling
**Goal**: Users can fully edit commit messages and merge messages, with comment lines visually distinct and preserved verbatim, conflict markers styled, and save/cancel hotkeys functional
**Depends on**: Phase 1
**Requirements**: CTX-03, CTX-04, CTX-05, CTX-06, EDIT-01, EDIT-02, EDIT-03, EDIT-04, EDIT-05, EDIT-06, EDIT-07, COMMIT-04, COMMIT-05, MERGE-01, MERGE-02, MERGE-03, PERF-02, PERF-03
**Success Criteria** (what must be TRUE):
  1. User can type, delete, navigate with arrow keys, move to line start/end, delete a line (Ctrl+U), and insert newlines — multiline messages work correctly
  2. Comment lines (lines starting with core.commentChar, default #) are displayed in a distinct style and cannot be edited or deleted
  3. On save, comment lines are written back byte-for-byte; the file is not corrupted
  4. Conflict markers (<<<<<<, ======, >>>>>>) in MERGE_MSG are styled and not editable; the resolved message between them is editable
  5. Ctrl+S saves and exits; Esc cancels and exits; rendering updates at 30+ FPS while typing
**Plans**: 3 plans

Plans:
- [ ] 02-01-PLAN.md — Document model, ContentLine enum, comment/marker parsing, serialization
- [ ] 02-02-PLAN.md — App refactor (Document + TextArea), per-line styled renderer, writer update
- [ ] 02-03-PLAN.md — Keyboard shortcuts (Ctrl+U/Z/Y/W/D), system clipboard (Ctrl+C/X/V), verification

### Phase 3: Commit Message Intelligence
**Goal**: The editor actively guides users toward well-formed commit messages via a real-time subject line counter, blank line enforcement, and an accessible hotkey help overlay
**Depends on**: Phase 2
**Requirements**: COMMIT-01, COMMIT-02, COMMIT-03, COMMIT-06, HELP-01, HELP-02, HELP-03, HELP-04
**Success Criteria** (what must be TRUE):
  1. Subject line shows a live character count; the count displays green if the line is <=50 chars, yellow for 51-72, and red for >72
  2. A blank line between subject and body is enforced or clearly suggested while editing
  3. Pressing Ctrl+H (or similar) opens a hotkey reference showing only the actions relevant to the current context (commit mode shows commit actions, not rebase actions)
  4. The hotkey reference does not interfere with editing and can be dismissed to resume editing without losing cursor position
**Plans**: 2 plans

Plans:
- [x] 03-01-PLAN.md — Subject line counter and blank line detection
- [x] 03-02-PLAN.md — Help overlay

### Phase 4: Rebase + Squash Modes
**Goal**: gitmedit handles interactive rebase and squash operations as a sequence.editor replacement — parsing git-rebase-todo into a structured table, supporting action cycling, and rendering squash commit logs as read-only context
**Depends on**: Phase 2
**Requirements**: REBASE-01, REBASE-02, REBASE-03, REBASE-04, REBASE-05, REBASE-06, SQUASH-01, SQUASH-02, SQUASH-03, SQUASH-04
**Success Criteria** (what must be TRUE):
  1. Running `git rebase -i` opens gitmedit with a structured table of commit actions (pick, squash, fixup, drop, etc.) rather than a free-form text buffer
  2. User can cycle through action types for any non-comment line using a hotkey (e.g., p→s→f→d→p); comment lines are protected
  3. Saving writes the rebase-todo file in exact git format; git rebase proceeds correctly without file corruption
  4. When editing a squash operation (SQUASH_MSG), the accumulated commit log is displayed as a styled read-only block and the combined commit message is editable
**Plans**: 3 plans

Plans:
- [ ] 04-01-PLAN.md — Rebase data model, parsing, action cycling, serialization
- [ ] 04-02-PLAN.md — Rebase table rendering, Tab/arrow key wiring, status bar
- [ ] 04-03-PLAN.md — Squash log parsing, dual-pane rendering, squash mode wiring

### Phase 5: Installation + Distribution
**Goal**: Users can install gitmedit with a single cargo command, find it in PATH, and configure git to use it for both standard commits and interactive rebase
**Depends on**: Phase 4
**Requirements**: INSTALL-01, INSTALL-02, INSTALL-03, INSTALL-04, INSTALL-05
**Success Criteria** (what must be TRUE):
  1. `cargo install --path .` completes without errors and the binary is available at `~/.cargo/bin/gitmedit`
  2. `git config --global core.editor gitmedit` and `git config --global sequence.editor gitmedit` configure git to invoke gitmedit for both commit messages and interactive rebase
  3. gitmedit respects whichever of core.editor or sequence.editor was used to invoke it and handles the corresponding file format correctly
**Plans**: 2 plans

Plans:
- [x] 05-01-PLAN.md — Cargo.toml metadata, LICENSE file, README.md with install/config instructions
- [ ] 05-02-PLAN.md — Install verification, PATH check, git editor integration test

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Git Contract + TUI Shell | 3/3 | Complete   | 2026-03-18 |
| 2. Text Editing + Comment Handling | 0/3 | Not started | - |
| 3. Commit Message Intelligence | 2/2 | Complete   | 2026-03-22 |
| 4. Rebase + Squash Modes | 3/3 | Complete   | 2026-03-23 |
| 5. Installation + Distribution | 1/2 | In Progress|  |

### Phase 6: Rebase view horizontal scrolling — implement left/right arrow navigation to view full commit subjects in rebase table

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 5
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd:plan-phase 6 to break down)
