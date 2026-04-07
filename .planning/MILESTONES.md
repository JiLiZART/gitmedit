# Milestones

## v1.0 MVP (Shipped: 2026-04-07)

**Phases completed:** 6 phases, 14 plans, 24 tasks
**Timeline:** 2026-03-18 → 2026-04-02 (15 days)
**Codebase:** 2,448 LOC Rust, 94 tests passing

**Key accomplishments:**

1. Git editor contract — file reads, TUI renders without alternate screen, Ctrl+S saves (exit 0), Esc cancels (exit 1), terminal always restored
2. Full text editing — Document model with comment/conflict parsing, nano-style shortcuts, system clipboard
3. Commit message intelligence — real-time subject counter (green/yellow/red), blank line enforcement, context-aware help overlay
4. Interactive rebase — structured table with action cycling (pick/squash/fixup/drop), color-coded rows, Tab/arrow navigation
5. Squash message editing — dual-pane with read-only commit log and editable combined message
6. Word-wrapped rebase subjects — long commit subjects wrap at word boundaries with variable-height scroll

### Known Gaps

| REQ-ID | Description | Status |
|--------|-------------|--------|
| IO-06 | No alternate screen | Regressed (terminal.rs uses EnterAlternateScreen) |
| EDIT-03 | Delete line (Ctrl+U) | Pending formal verification |
| EDIT-07 | Undo/redo | Pending formal verification |
| PERF-02 | 30+ FPS rendering | Pending formal verification |
| PERF-03 | No lag on large files | Pending formal verification |
| INSTALL-05 | Respects both editor configs | Pending |

See `milestones/v1.0-MILESTONE-AUDIT.md` for full audit report.

---
