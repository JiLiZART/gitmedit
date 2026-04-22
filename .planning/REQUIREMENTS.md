# Requirements: gitmedit

**Defined:** 2026-04-07
**Core Value:** Fast, distraction-free git editor that feels like nano's simplicity but understands git's context

## v1.1 Requirements

Requirements for v1.1 milestone. Each maps to roadmap phases.

### Editor Core

- [ ] **EDIT-10**: All lines are editable by default (no read-only comment protection)
- [ ] **EDIT-11**: Editor uses ratatui_textarea default behavior for all text editing
- [x] **IO-06**: Editor renders inline without alternate screen (nano-style)
- [x] **IO-07**: TerminalGuard::Drop uses safe error handling (no unwrap/panic)

### UI Chrome

- [ ] **UI-01**: Top header bar displays folder/project name in all modes
- [ ] **UI-02**: Header bar displays current filename (like nano title bar)
- [ ] **UI-03**: Bottom command bar shows available key shortcuts (^S Save, Esc Cancel, etc.)
- [ ] **UI-04**: Command bar adapts to current mode (commit vs rebase vs merge)
- [ ] **UI-05**: Merge mode toolbar parses MERGE_MSG comment block for conflict file list
- [ ] **UI-06**: Merge mode toolbar displays affected/conflict files in status area

### Standalone Commit

- [ ] **COMMIT-10**: User can run `gitmedit` with no args inside a git repo to open commit editor
- [ ] **COMMIT-11**: Editor shows empty message area for writing commit message
- [ ] **COMMIT-12**: Ctrl+S commits staged changes via `git commit -F <tmpfile>` and exits
- [ ] **COMMIT-13**: Editor shows error and exits if no changes are staged
- [ ] **COMMIT-14**: Git commit errors (e.g., hook failure) are displayed to user before exit

### Rebase Enhancements

- [ ] **REBASE-10**: User can move rebase todo lines up/down (Alt+Up/Alt+Down)
- [ ] **REBASE-11**: Cursor follows the moved line after reordering
- [ ] **REBASE-12**: selectable_indices are regenerated after each reorder operation
- [ ] **REBASE-13**: User can edit arguments on exec lines in rebase mode

### Cross-Platform

- [ ] **PLAT-01**: Editor renders correctly on macOS Terminal and iTerm2
- [ ] **PLAT-02**: Editor renders correctly on Windows Terminal
- [ ] **PLAT-03**: Key events handled correctly on Windows (no double-fire)

## v1.2+ Requirements

Deferred to future release. Tracked but not in current roadmap.

### Configuration

- **CONFIG-01**: User can customize key bindings via TOML config file
- **CONFIG-02**: Config file located at ~/.config/gitmedit/config.toml (XDG-compliant)
- **CONFIG-03**: Missing config file uses sensible defaults

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Custom hotkey configuration | Deferred to v1.2 — all input handlers must be stable first |
| git2/gix crate integration | Binary size cost not justified for single subprocess call |
| Interactive staging (git add -p) | gitmedit is an editor, not a staging tool |
| Vim/Emacs keybindings | Standard shortcuts only for simplicity |
| Plugin system | Single focused tool, not extensible |
| Graphical UI | Terminal only |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| IO-06 | Phase 7 | Complete |
| IO-07 | Phase 7 | Complete |
| EDIT-10 | Phase 8 | Pending |
| EDIT-11 | Phase 8 | Pending |
| UI-01 | Phase 9 | Pending |
| UI-02 | Phase 9 | Pending |
| UI-03 | Phase 9 | Pending |
| UI-04 | Phase 9 | Pending |
| UI-05 | Phase 10 | Pending |
| UI-06 | Phase 10 | Pending |
| REBASE-10 | Phase 10 | Pending |
| REBASE-11 | Phase 10 | Pending |
| REBASE-12 | Phase 10 | Pending |
| REBASE-13 | Phase 10 | Pending |
| COMMIT-10 | Phase 11 | Pending |
| COMMIT-11 | Phase 11 | Pending |
| COMMIT-12 | Phase 11 | Pending |
| COMMIT-13 | Phase 11 | Pending |
| COMMIT-14 | Phase 11 | Pending |
| PLAT-01 | Phase 12 | Pending |
| PLAT-02 | Phase 12 | Pending |
| PLAT-03 | Phase 12 | Pending |

**Coverage:**
- v1.1 requirements: 22 total
- Mapped to phases: 22
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-07*
*Last updated: 2026-04-07 after roadmap creation*
