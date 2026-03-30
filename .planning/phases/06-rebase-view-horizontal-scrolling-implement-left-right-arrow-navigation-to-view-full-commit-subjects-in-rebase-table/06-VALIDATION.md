---
phase: 6
slug: rebase-view-horizontal-scrolling-implement-left-right-arrow-navigation-to-view-full-commit-subjects-in-rebase-table
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-30
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`#[test]`) + `cargo test` |
| **Config file** | Cargo.toml (already configured) |
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

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 1 | wrap_subject pure fn | unit | `cargo test wrap_subject` | ❌ W0 | ⬜ pending |
| 06-01-02 | 01 | 1 | scroll offset with multi-line rows | unit | `cargo test rebase_scroll` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Unit tests for `wrap_subject()` function — word-boundary wrapping, UTF-8 safety, edge cases
- [ ] Unit tests for scroll offset accumulator with multi-line rows

*Existing test infrastructure (cargo test) covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visual wrapping renders correctly in rebase table | Display quality | Requires visual terminal inspection | Open `git rebase -i` with long commit subjects, verify wrapped text aligns in subject column |
| Scroll behavior with mixed single/multi-line rows | Scroll correctness | Terminal visual behavior | Scroll through 20+ entry rebase-todo with long subjects, verify no overwrite of status bar |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
