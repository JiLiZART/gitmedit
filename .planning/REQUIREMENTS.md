# Requirements: gitmedit

**Defined:** 2026-03-18
**Core Value:** Provide a fast, distraction-free git editor that feels like nano's simplicity but understands git's context (merges, rebases, squashes) without bloat.

## v1 Requirements

### File I/O & Terminal

- [x] **IO-01**: Editor reads file from first command-line argument
- [x] **IO-02**: Editor writes edited content back to the same file on save
- [x] **IO-03**: Terminal is safely restored to normal mode even if editor panics
- [x] **IO-04**: Editor exits with code 0 on successful save
- [x] **IO-05**: Editor exits with code 1 on cancel or error
- [x] **IO-06**: Editor does NOT use alternate screen (matches nano behavior)

### Context Detection

- [x] **CTX-01**: Editor detects file type from path argument (COMMIT_EDITMSG, MERGE_MSG, git-rebase-todo, SQUASH_MSG)
- [x] **CTX-02**: Detected context determines UI mode (text editor vs structured rebase mode)
- [x] **CTX-03**: Editor reads `core.commentChar` config at startup (default: #)
- [x] **CTX-04**: Comment lines are parsed and stored separately from editable content
- [x] **CTX-05**: Comment lines are displayed but not editable
- [x] **CTX-06**: Comment lines are preserved byte-for-byte on file write

### Text Editing (Commit Mode)

- [x] **EDIT-01**: User can insert/delete characters anywhere in the message
- [x] **EDIT-02**: User can move cursor with arrow keys
- [ ] **EDIT-03**: User can delete line with Ctrl+U
- [x] **EDIT-04**: User can move to start/end of line (Home/End)
- [x] **EDIT-05**: User can create new lines (multiline messages supported)
- [x] **EDIT-06**: Text wraps at terminal width (no horizontal scroll needed)
- [ ] **EDIT-07**: Undo/redo work for text edits

### Commit Message Features

- [x] **COMMIT-01**: Subject line character counter displays (real-time)
- [x] **COMMIT-02**: Subject line shows green if ≤50 chars, yellow if 50-72, red if >72
- [x] **COMMIT-03**: Blank line between subject and body is enforced/suggested
- [x] **COMMIT-04**: User can save with Ctrl+S
- [x] **COMMIT-05**: User can cancel with Esc
- [x] **COMMIT-06**: Hotkey help shows on Ctrl+H (or similar)

### Merge Conflict Handling

- [x] **MERGE-01**: Conflict markers (<<<<<<, ======, >>>>>>) are detected and styled
- [x] **MERGE-02**: Conflict markers are not editable (treated as comments)
- [x] **MERGE-03**: User can edit the resolved message between markers

### Rebase Todo Mode

- [x] **REBASE-01**: Rebase todo file is parsed into lines with actions (pick, squash, fixup, drop, etc.)
- [x] **REBASE-02**: Rebase mode displays lines in a structured table (not free-form text)
- [x] **REBASE-03**: User can cycle through action types with hotkey (e.g., p→s→f→d→p)
- [x] **REBASE-04**: Non-comment lines can have their action changed
- [x] **REBASE-05**: Comment lines and order are preserved on save
- [x] **REBASE-06**: Save writes rebase todo back in exact git format

### Squash Detection

- [ ] **SQUASH-01**: Squash mode detects when editing a squash operation file
- [ ] **SQUASH-02**: Squash mode displays the original commit log (read-only)
- [ ] **SQUASH-03**: Squash mode allows editing the combined commit message
- [ ] **SQUASH-04**: Squash commits list is highlighted and protected from editing

### Hotkey Help & Display

- [x] **HELP-01**: Hotkey reference is available (Ctrl+H or similar)
- [x] **HELP-02**: Hotkey reference shows only relevant actions for current mode
- [x] **HELP-03**: Hotkey reference does not interfere with message editing
- [x] **HELP-04**: Help can be dismissed and editing resumes

### Performance

- [x] **PERF-01**: Editor starts in <100ms on typical hardware
- [ ] **PERF-02**: Rendering updates happen at 30+ FPS when typing
- [ ] **PERF-03**: No noticeable lag on large files (>10KB commit messages)

### Installation & Configuration

- [ ] **INSTALL-01**: Binary can be installed with `cargo install --path crates/gitmedit`
- [ ] **INSTALL-02**: Binary is available in PATH after installation
- [ ] **INSTALL-03**: User can set as `core.editor` with `git config --global core.editor gitmedit`
- [ ] **INSTALL-04**: User can set as `sequence.editor` with `git config --global sequence.editor gitmedit`
- [ ] **INSTALL-05**: Editor respects both `core.editor` and `sequence.editor` configs

## v2 Requirements

### Enhanced Editing

- **EDIT-08**: Reorder rebase commit lines (move up/down)
- **EDIT-09**: Delete entire rebase commits
- **EDIT-10**: Search/replace in message

### Terminal Features

- **TERM-01**: Handle terminal resize gracefully
- **TERM-02**: Support for 256-color terminals
- **TERM-03**: Windows terminal support (if not in v1)

### Configuration

- **CONFIG-01**: User can customize hotkeys (config file)
- **CONFIG-02**: User can customize color scheme
- **CONFIG-03**: User can set startup mode preference (explicit vs auto-detect)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Vim keybindings | Undermines "standard shortcuts only" design; nano-style is the goal |
| Emacs keybindings | Same as vim — keep it simple |
| Plugin system | Single focused tool, not extensible |
| Syntax highlighting | Minimal UI; commit messages don't need color |
| Tree/file browser | Not applicable to git editor use case |
| Configuration files | Sensible defaults; avoid complexity |
| Undo/redo UI | Provided transparently via hotkeys, not as visible mode |
| `core.commentChar = auto` detection | Support fixed `#`, document as v1 limitation |
| Line reordering in rebase | High interaction complexity; defer to v2 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| IO-01 | Phase 1 | Complete |
| IO-02 | Phase 1 | Complete |
| IO-03 | Phase 1 | Complete |
| IO-04 | Phase 1 | Complete |
| IO-05 | Phase 1 | Complete |
| IO-06 | Phase 1 | Complete |
| CTX-01 | Phase 1 | Complete |
| CTX-02 | Phase 1 | Complete |
| PERF-01 | Phase 1 | Complete |
| CTX-03 | Phase 2 | Complete |
| CTX-04 | Phase 2 | Complete |
| CTX-05 | Phase 2 | Complete |
| CTX-06 | Phase 2 | Complete |
| EDIT-01 | Phase 2 | Complete |
| EDIT-02 | Phase 2 | Complete |
| EDIT-03 | Phase 2 | Pending |
| EDIT-04 | Phase 2 | Complete |
| EDIT-05 | Phase 2 | Complete |
| EDIT-06 | Phase 2 | Complete |
| EDIT-07 | Phase 2 | Pending |
| COMMIT-04 | Phase 2 | Complete |
| COMMIT-05 | Phase 2 | Complete |
| MERGE-01 | Phase 2 | Complete |
| MERGE-02 | Phase 2 | Complete |
| MERGE-03 | Phase 2 | Complete |
| PERF-02 | Phase 2 | Pending |
| PERF-03 | Phase 2 | Pending |
| COMMIT-01 | Phase 3 | Complete |
| COMMIT-02 | Phase 3 | Complete |
| COMMIT-03 | Phase 3 | Complete |
| COMMIT-06 | Phase 3 | Complete |
| HELP-01 | Phase 3 | Complete |
| HELP-02 | Phase 3 | Complete |
| HELP-03 | Phase 3 | Complete |
| HELP-04 | Phase 3 | Complete |
| REBASE-01 | Phase 4 | Complete |
| REBASE-02 | Phase 4 | Complete |
| REBASE-03 | Phase 4 | Complete |
| REBASE-04 | Phase 4 | Complete |
| REBASE-05 | Phase 4 | Complete |
| REBASE-06 | Phase 4 | Complete |
| SQUASH-01 | Phase 4 | Pending |
| SQUASH-02 | Phase 4 | Pending |
| SQUASH-03 | Phase 4 | Pending |
| SQUASH-04 | Phase 4 | Pending |
| INSTALL-01 | Phase 5 | Pending |
| INSTALL-02 | Phase 5 | Pending |
| INSTALL-03 | Phase 5 | Pending |
| INSTALL-04 | Phase 5 | Pending |
| INSTALL-05 | Phase 5 | Pending |

**Coverage:**
- v1 requirements: 50 total
- Mapped to phases: 50/50
- Unmapped: 0

---
*Requirements defined: 2026-03-18*
*Last updated: 2026-03-18 — traceability populated after roadmap creation*
