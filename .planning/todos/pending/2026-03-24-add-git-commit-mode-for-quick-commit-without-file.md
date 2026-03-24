---
created: 2026-03-24T11:43:54.916Z
title: Add git commit mode for quick-commit without file
area: features
files:
  - src/main.rs
  - src/context.rs
  - src/app.rs
---

## Problem

Currently gitmedit requires a file argument to run. Users must go through the normal `git commit` flow. There's an opportunity to add a new mode: when gitmedit is invoked without arguments AND inside a git repository, it enters a "commit quick mode" that:
- Opens an empty textarea for commit message input
- Shows context-aware hotkeys (Ctrl+S for "Commit", Esc for "exit")
- After user enters message and presses Ctrl+S, it passes the text to `git commit -m "text here"`
- This provides a faster, more intuitive way to create commits without opening a full editor

This is a **v2+ feature idea**, not required for v1.0 (Phase 5 completes v1.0 scope).

## Solution

1. **Detect no-file mode:** In `main.rs`, check if `std::env::args().nth(1)` is None
2. **Check if in git repo:** Use existing git context detection or `git rev-parse --git-dir`
3. **New GitContext variant:** Add `GitContext::QuickCommit` to context.rs enum
4. **App behavior changes:**
   - Empty textarea initialized
   - Help text shows "Ctrl+S (Commit) Esc (exit)" instead of standard editor help
   - Event loop detects Ctrl+S and passes all textarea content to `git commit -m`
   - Exit with code 0 on success, code 1 on abort/error
5. **Implementation order:** After Phase 5 (v1.0), consider for Phase 6 (v1.1 features)

### Acceptance Criteria

- [ ] `gitmedit` (no args, in git repo) enters commit mode with empty textarea
- [ ] Help overlay shows "Commit" and "exit" labels (not standard edit commands)
- [ ] Ctrl+S invokes `git commit -m "message content"`
- [ ] Non-zero git commit exit codes are propagated to gitmedit exit code
- [ ] `gitmedit` (no args, NOT in git repo) shows error: "Not in a git repository"
- [ ] Existing file-based modes (commit, rebase, merge) are unaffected
