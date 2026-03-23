---
phase: 04-rebase-squash-modes
plan: 01
subsystem: editor
tags: [rust, rebase, git-rebase-todo, parsing, serialization, state-machine]

# Dependency graph
requires:
  - phase: 02-text-editing-comment-handling
    provides: Document struct, ContentLine enum, parse/serialize patterns used as reference
provides:
  - RebaseAction enum with Pick/Squash/Fixup/Drop/Exec variants and cycle() rotation
  - RebaseLine enum with Action{action,hash,subject} and Comment variants
  - parse_action() accepting both long and short forms
  - classify_rebase_line() with comment detection, exec special case, malformed fallback
  - parse_rebase_todo() with trailing-newline handling
  - serialize_rebase_todo() normalizing to long-form action names
  - App extended with rebase_lines, selected_rebase_idx, selectable_indices
  - CycleRebaseAction, MoveRebaseDown, MoveRebaseUp actions
affects:
  - 04-02 (rendering): needs rebase_lines, selected_rebase_idx, selectable_indices accessors
  - 04-03 (squash): builds on rebase cycling logic

# Tech tracking
tech-stack:
  added: []
  patterns:
    - TDD RED/GREEN for Rust unit tests within binary crate (cargo test --bin)
    - Trailing-newline handling: skip phantom empty token from split('\n') - same as Document::parse
    - splitn(3, ' ') for parsing "action hash subject" lines with multi-word subjects
    - Exec special case: no hash, entire remainder is subject
    - Malformed lines fall back to Comment (no panic)
    - selectable_indices as pre-computed index list for O(1) navigation

key-files:
  created: []
  modified:
    - src/document.rs
    - src/app.rs

key-decisions:
  - "parse_rebase_todo returns empty Vec for empty string (not single Comment) — avoids edge case in navigator"
  - "selectable_indices pre-computed at parse time, not filtered on every navigation — O(1) navigation"
  - "Exec cycle() returns self (Exec) — exec lines are not user-cyclable actions"
  - "cargo test --lib does not work for binary crates; plan verification command corrected to cargo test --bin gitmedit"

patterns-established:
  - "Exec line format: action=Exec, hash empty string, subject=entire remainder after 'exec '"
  - "Abbreviated action forms (p/s/f/d/x) accepted on parse; long forms always emitted on serialize"
  - "MoveRebaseDown/Up clamp at bounds (no wrap-around)"

requirements-completed:
  - REBASE-01
  - REBASE-03
  - REBASE-04
  - REBASE-05
  - REBASE-06

# Metrics
duration: 4min
completed: 2026-03-23
---

# Phase 4 Plan 1: Rebase Data Model Summary

**RebaseLine/RebaseAction enums with parse_action/parse_rebase_todo/serialize_rebase_todo in document.rs, plus App rebase state (rebase_lines, selectable_indices, CycleRebaseAction) in app.rs**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-23T11:41:45Z
- **Completed:** 2026-03-23T11:45:29Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- RebaseAction enum with cycle() rotation (Pick->Squash->Fixup->Drop->Pick, Exec no-op) and as_str() long-form output
- RebaseLine enum parsed from git-rebase-todo format with comment/exec/malformed fallback handling
- App struct extended with rebase state: rebase_lines, selectable_indices (pre-computed), selected_rebase_idx
- CycleRebaseAction, MoveRebaseDown, MoveRebaseUp actions with clamping navigation
- serialized_content() branches on GitContext::Rebase to call serialize_rebase_todo()
- 30 new unit tests (20 in document.rs, 10 in app.rs); 77 total passing, zero regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: RebaseLine enum, RebaseAction enum, parsing, serialization** - `ff3feea` (feat)
2. **Task 2: App rebase state management and CycleRebaseAction** - `94f519e` (feat)

**Plan metadata:** (docs commit follows)

_Note: TDD tasks have integrated RED+GREEN in single commit per task_

## Files Created/Modified

- `src/document.rs` - Added RebaseAction, RebaseLine, parse_action(), classify_rebase_line(), parse_rebase_todo(), serialize_rebase_todo() with 20 unit tests
- `src/app.rs` - Extended Action enum, App struct, apply() handler, serialized_content() override, new accessors; 10 unit tests

## Decisions Made

- `cargo test --lib` does not work for binary crates (Cargo requires `--bin gitmedit`); plan verification command is wrong but outcome is identical
- parse_rebase_todo returns empty Vec for empty input rather than a single Comment — cleaner for the navigator
- selectable_indices pre-computed at parse time for O(1) tab-navigation without filtering on every keypress
- Exec cycle() is a no-op — exec lines execute shell commands, not git operations; cycling makes no semantic sense

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected cargo test invocation for binary crate**
- **Found during:** Task 1 verification
- **Issue:** Plan verification command `cargo test --lib document::tests` fails — project is a binary crate with no lib target
- **Fix:** Used `cargo test --bin gitmedit` (equivalent result, all tests run)
- **Files modified:** None (test execution only)
- **Verification:** 77 tests pass
- **Committed in:** ff3feea (inline fix, no separate commit needed)

---

**Total deviations:** 1 auto-fixed (Rule 1 - verification command for binary crate)
**Impact on plan:** No functional change. Test coverage is identical.

## Issues Encountered

None beyond the cargo test invocation correction above.

## Next Phase Readiness

- Plan 02 (rendering) has all accessors it needs: rebase_lines(), selected_rebase_idx(), selectable_indices()
- Plan 03 (squash) can rely on cycle() and CycleRebaseAction being fully tested
- No blockers

---
*Phase: 04-rebase-squash-modes*
*Completed: 2026-03-23*
