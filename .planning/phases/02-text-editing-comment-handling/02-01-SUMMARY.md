---
phase: 02-text-editing-comment-handling
plan: 01
subsystem: document-model
tags: [rust, document-model, comment-detection, git-config, serialization]

# Dependency graph
requires:
  - phase: 01-git-contract-tui-shell
    provides: App struct and GitContext enum used to understand existing architecture boundaries
provides:
  - ContentLine enum with Content/Comment/ConflictMarker variants
  - Document struct with parse/serialize/editable_lines API
  - read_comment_char() reading git config core.commentchar with '#' fallback
  - arboard 3.6 clipboard dependency in Cargo.toml
affects: [02-02, 02-03, 03, 04, 05]

# Tech tracking
tech-stack:
  added: [arboard 3.6]
  patterns:
    - trailing-newline-aware line splitting (drop phantom empty token after '\n')
    - editable_index indirection for mapping textarea rows to full line indices
    - serialize() walks original line order using content_idx counter for Content replacement

key-files:
  created: [src/document.rs]
  modified: [Cargo.toml, src/main.rs]

key-decisions:
  - "Trailing '\n' in parse() is consumed: the phantom empty token from split() is discarded, serialize() re-emits '\\n' after every stored line — this is the correct round-trip invariant"
  - "comment_only_file with trailing newline produces 0 editable lines (not 1) because trailing empty token is dropped"
  - "6-char conflict marker prefix matching catches both 6-char and 7-char git variants (<<<<<<< HEAD)"
  - "cargo test --lib fails on binary crates; tests run via cargo test --bin gitmedit or cargo test"

patterns-established:
  - "Document is the central data model — all subsequent rendering and editing plans work through it"
  - "editable_index maps TextArea row N -> lines[N] index, keeping non-editable lines transparent to the editor"

requirements-completed: [CTX-03, CTX-04, CTX-06, MERGE-01]

# Metrics
duration: 5min
completed: 2026-03-20
---

# Phase 02 Plan 01: Document Data Model Summary

**ContentLine enum + Document struct with trailing-newline-aware parse/serialize, git config comment char detection, and 16 passing unit tests**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-20T12:30:05Z
- **Completed:** 2026-03-20T12:34:37Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Created `src/document.rs` with ContentLine enum (Content/Comment/ConflictMarker), Document struct, parse/serialize, and comment char detection
- Implemented `read_comment_char()` calling `git config --get core.commentchar` with `#` fallback for all error paths including `auto` value
- Added arboard 3.6 clipboard dependency to Cargo.toml
- All 16 unit tests pass covering classification, parse, editable index mapping, roundtrip serialization, and edge cases

## Task Commits

1. **Task 1 + Task 2: ContentLine enum, Document struct, arboard dep, 16 unit tests** - `3292c70` (feat)

Note: Both tasks modified `src/document.rs` exclusively. The implementation and test suite were committed atomically in one commit.

## Files Created/Modified

- `src/document.rs` - ContentLine enum, Document struct, classify_line(), read_comment_char(), parse(), serialize(), editable_lines(), full_row_for_editable(), update_content_line(), 16 unit tests
- `Cargo.toml` - Added arboard = "3.6" to [dependencies]
- `src/main.rs` - Added `mod document;` declaration

## Decisions Made

- **Trailing newline handling:** `parse()` drops the phantom empty token that `split('\n')` produces after a trailing `'\n'`. `serialize()` always appends `'\n'` after every stored line. This ensures `parse(serialize(editable_lines))` round-trips the original bytes byte-for-byte.
- **Binary crate test invocation:** The project has no lib target, so `cargo test --lib` errors. Tests run via `cargo test document::tests` (or `cargo test` for all). The plan's `<verify>` command `cargo test --lib` was corrected to `cargo test`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Trailing newline caused double-newline in roundtrip serialization**

- **Found during:** Task 2 (test_roundtrip_preserves_comments and related tests failing)
- **Issue:** `"subject\n".split('\n')` produces `["subject", ""]`. Serializing each element with appended `'\n'` produced `"subject\n\n"` instead of `"subject\n"`.
- **Fix:** When `raw.ends_with('\n')` and there is more than one token, drop the final empty token before building the `lines` Vec. Updated 4 tests that were counting the phantom trailing Content line.
- **Files modified:** src/document.rs (parse() + test expectations)
- **Verification:** All 16 tests pass including the 3 roundtrip tests
- **Committed in:** 3292c70

**2. [Rule 1 - Bug] `cargo test --lib` fails on binary crate**

- **Found during:** Task 1 verify step
- **Issue:** The plan's `<automated>` verify command uses `cargo test --lib document::tests` but the project is a binary crate with no lib target — the command exits with "no library targets found".
- **Fix:** Used `cargo test document::tests` instead.
- **Files modified:** None (command-line only)
- **Verification:** `cargo test document::tests` exits 0, 16 passed

---

**Total deviations:** 2 auto-fixed (2 Rule 1 bugs)
**Impact on plan:** Both fixes are necessary for correctness. No scope creep.

## Issues Encountered

None beyond the two auto-fixed deviations above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `src/document.rs` is complete and stable. Plans 02-02 and 02-03 can import `Document` and `ContentLine` directly.
- The `update_content_line()` and `editable_count()` methods are unused warnings now — they are forward-facing APIs for 02-02's editor integration.

---
*Phase: 02-text-editing-comment-handling*
*Completed: 2026-03-20*
