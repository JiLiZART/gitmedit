---
phase: 4
slug: rebase-squash-modes
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in + cargo test |
| **Config file** | Cargo.toml (existing) |
| **Quick run command** | `cargo test --lib document::tests --lib context::tests` |
| **Full suite command** | `cargo test --lib` |
| **Estimated runtime** | ~2s (quick), ~5s (full) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib --quiet`
- **After every plan wave:** Run `cargo test --lib` (full suite)
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Requirement | Behavior | Test Type | Automated Command | File Exists | Status |
|-------------|----------|-----------|-------------------|-------------|--------|
| REBASE-01 | Parse rebase-todo lines into RebaseLine enum with action+hash+subject | unit | `cargo test --lib test_classify_rebase_line` | ❌ Wave 0 | ⬜ pending |
| REBASE-02 | Table widget renders rebase lines with action, hash, subject columns | integration | `cargo test --lib test_render_rebase_table` | ❌ Wave 0 | ⬜ pending |
| REBASE-03 | Tab key cycles actions (pick→squash→fixup→drop→pick) | unit | `cargo test --lib test_action_cycle` | ❌ Wave 0 | ⬜ pending |
| REBASE-04 | Non-comment lines action field updated by Tab handler | unit | `cargo test --lib test_cycle_rebase_action` | ❌ Wave 0 | ⬜ pending |
| REBASE-05 | Comment lines and rebase line order preserved during parse/serialize roundtrip | unit | `cargo test --lib test_roundtrip_rebase_todo` | ❌ Wave 0 | ⬜ pending |
| REBASE-06 | Serialize reconstructs rebase-todo in exact git format `{action} {hash} {subject}` | unit | `cargo test --lib test_serialize_rebase_todo` | ❌ Wave 0 | ⬜ pending |
| SQUASH-01 | SQUASH_MSG file detected as GitContext::Squash | unit | `cargo test --lib test_detect_squash_context` | ✅ Phase 1 | ⬜ pending |
| SQUASH-02 | Parse "This is a combination of N commits" block as read-only header | unit | `cargo test --lib test_parse_squash_log` | ❌ Wave 0 | ⬜ pending |
| SQUASH-03 | TextArea positioned below commit log; user edits only message portion | integration | `cargo test --lib test_render_squash_dual_pane` | ❌ Wave 0 | ⬜ pending |
| SQUASH-04 | Commit log rendered with distinct background; prevents editing (protected) | unit | `cargo test --lib test_squash_log_protected` | ❌ Wave 0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/document.rs` — add RebaseLine enum and classify_rebase_line()
- [ ] `src/document.rs` — add detect_squash_context() and parse_squash_log()
- [ ] `src/document.rs` — extend Document struct to hold rebase_lines: Vec<RebaseLine> when context is Rebase
- [ ] `src/app.rs` — extend Action enum with CycleRebaseAction
- [ ] `src/app.rs` — add selected_rebase_idx: usize field; cycle_rebase_action() method
- [ ] `src/renderer.rs` — add render_rebase_table() function
- [ ] `src/renderer.rs` — add render_squash_mode() function with dual-pane layout
- [ ] `src/main.rs` — map Tab key to CycleRebaseAction in event loop (only in Rebase context)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| User can press Tab in rebase mode and see action cycle visually | REBASE-03 | UI interaction requires visual inspection | 1. Create interactive rebase: `git rebase -i HEAD~5`; 2. On first line, press Tab repeatedly; 3. Verify action cycles: pick→squash→fixup→drop→pick |
| Commit log in squash mode is visually distinct and cannot be selected/edited | SQUASH-04 | Visual styling and interaction feedback requires human inspection | 1. Create squash scenario: `git rebase -i HEAD~2`, mark first as pick, second as squash; 2. Proceed to SQUASH_MSG edit; 3. Verify log section has different background; 4. Try to edit log lines; 5. Verify cursor cannot enter log area |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
