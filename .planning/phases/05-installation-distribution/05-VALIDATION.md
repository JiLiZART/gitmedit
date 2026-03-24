---
phase: 05
slug: installation-distribution
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-24
---

# Phase 05 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test (`#[test]`) |
| **Config file** | none (cargo built-in) |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo publish --dry-run` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo publish --dry-run`
- **Before `/gsd:verify-work`:** Full suite must be green, plus manual smoke test (cargo install --path .)
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | INSTALL-01, INSTALL-02, INSTALL-03, INSTALL-04 | smoke | `cd /Users/jilizart/Projects/Github/gitmedit && grep -q "description" Cargo.toml && grep -q "license" Cargo.toml` | ✅ | ⬜ pending |
| 05-01-02 | 01 | 1 | INSTALL-01, INSTALL-02, INSTALL-03, INSTALL-04 | smoke | `test -f README.md && grep -q "## Installation" README.md && cargo publish --dry-run 2>&1 \| head -5` | ✅ | ⬜ pending |
| 05-02-01 | 02 | 2 | INSTALL-01, INSTALL-02, INSTALL-03, INSTALL-04 | smoke | `cargo install --path . --force 2>&1 && which gitmedit && git config --global core.editor gitmedit` | ✅ | ⬜ pending |
| 05-02-02 | 02 | 2 | INSTALL-05 | unit | `cargo test context -- --nocapture` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. All 86 existing tests pass, including context.rs tests that verify INSTALL-05. No new test files needed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| git invokes gitmedit for commit messages | INSTALL-03 | Requires live git repo with commits to trigger core.editor | 1. `git config --global core.editor gitmedit`<br/>2. `cd /tmp && git init test-repo && cd test-repo`<br/>3. `echo 'test' > file.txt && git add file.txt`<br/>4. `git commit` — verify gitmedit opens and saves message correctly |
| git invokes gitmedit for interactive rebase | INSTALL-04 | Requires live git repo with multiple commits to trigger sequence.editor | 1. `git config --global sequence.editor gitmedit`<br/>2. In a git repo with 2+ commits: `git rebase -i HEAD~2`<br/>3. Verify gitmedit opens rebase-todo and can cycle actions, save exits with code 0 |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (none — all existing)
- [x] No watch-mode flags
- [x] Feedback latency < 10s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-03-24
