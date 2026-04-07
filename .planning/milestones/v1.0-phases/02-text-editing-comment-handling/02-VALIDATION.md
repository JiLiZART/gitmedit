---
phase: 02
slug: text-editing-comment-handling
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-20
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`cargo test`) |
| **Config file** | none — inline `#[cfg(test)]` modules |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CTX-03 | comment_char defaults to `#` when git config absent | unit | `cargo test --lib document::tests::comment_char_default` | ❌ Wave 1 |
| CTX-04 | Comment lines parsed into Comment variant | unit | `cargo test --lib document::tests::parse_comment_lines` | ❌ Wave 1 |
| CTX-05 | Comment lines not present in `editable_lines()` | unit | `cargo test --lib document::tests::editable_lines_excludes_comments` | ❌ Wave 2 |
| CTX-06 | Roundtrip: parse then serialize preserves bytes | unit | `cargo test --lib writer::tests::roundtrip_preserves_comments` | ❌ Wave 2 |
| EDIT-01 | Insert char advances cursor and modifies content | unit | `cargo test --lib app::tests::insert_char_modifies_content` | ❌ Wave 2 |
| EDIT-02 | Arrow keys move cursor | unit | `cargo test --lib app::tests::arrow_keys_move_cursor` | ❌ Wave 2 |
| EDIT-03 | Ctrl+U deletes current line | unit | `cargo test --lib app::tests::ctrl_u_deletes_line` | ❌ Wave 3 |
| EDIT-04 | Home/End moves to line start/end | unit | `cargo test --lib app::tests::home_end_keys` | ❌ Wave 2 |
| EDIT-05 | Enter inserts newline | unit | `cargo test --lib app::tests::enter_inserts_newline` | ❌ Wave 2 |
| EDIT-07 | Ctrl+Z undoes last edit | unit | `cargo test --lib app::tests::ctrl_z_undoes` | ❌ Wave 3 |
| MERGE-01 | Conflict markers detected and stored as ConflictMarker | unit | `cargo test --lib document::tests::conflict_markers_detected` | ❌ Wave 1 |
| MERGE-02 | ConflictMarker lines not present in editable_lines() | unit | `cargo test --lib document::tests::conflict_markers_not_editable` | ❌ Wave 2 |
| MERGE-03 | Conflict markers preserved verbatim in serialization | unit | `cargo test --lib writer::tests::roundtrip_preserves_markers` | ❌ Wave 2 |
| COMMIT-04 | Ctrl+S saves and exits (Phase 1 - no change) | manual | (Already tested in Phase 1 execution) | ✅ Phase 1 |
| COMMIT-05 | Esc cancels and exits (Phase 1 - no change) | manual | (Already tested in Phase 1 execution) | ✅ Phase 1 |
| PERF-02 | Rendering >30 FPS when typing | manual | Start editor, type rapidly, visually confirm no lag | manual only |
| PERF-03 | Load 10KB file without noticeable lag | manual | `printf '#%s\n' {1..500} > /tmp/large && gitmedit /tmp/large` | manual only |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/document.rs` — ContentLine enum, Document struct, parse(), editable_lines(), editable_index mapping, serialize() — covers CTX-03, CTX-04, CTX-05, CTX-06, MERGE-01, MERGE-02, MERGE-03
- [ ] `src/parser.rs` — read_comment_char() subprocess call, Parser::parse() for comment line detection — covers CTX-03, CTX-04
- [ ] `src/app.rs` test module — test coverage for EDIT-01, EDIT-02, EDIT-04, EDIT-05 — covers basic text editing operations
- [ ] `src/main.rs` — verify Ctrl+S/Esc still work after refactoring — covers COMMIT-04, COMMIT-05
- [ ] `src/writer.rs` test module — roundtrip tests for comment and conflict marker preservation — covers CTX-06, MERGE-03

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real-time rendering performance (>30 FPS) | PERF-02 | Frame timing requires visual/interactive feedback; cannot be automated in headless test | Start gitmedit with a moderately-sized file, type rapidly, observe no visual lag or stutter in rendering |
| Large file handling (10KB+) | PERF-03 | Requires spawning the full editor process and observing real-world performance | Generate a 10KB file (`printf '#%s\n' {1..500} > /tmp/test.txt`), open with gitmedit, navigate and edit, confirm responsive |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify command or Wave 0 test stubs
- [ ] No 3 consecutive tasks without automated verification
- [ ] Wave 0 test files created before Wave 1 execution
- [ ] `cargo test --lib` passes after each task commit
- [ ] `cargo test` passes after each wave
- [ ] Manual performance tests (PERF-02, PERF-03) completed before phase sign-off
- [ ] No watch-mode or indefinite-loop tests
- [ ] Feedback latency <5 seconds
- [ ] `nyquist_compliant: true` confirmed in frontmatter

**Approval:** pending (set to approved once Wave 0 tests are written and passing)

---

*Phase: 02-text-editing-comment-handling*
*Created: 2026-03-20*
