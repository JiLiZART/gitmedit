---
phase: 7
slug: foundations-fix
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-16
---

# Phase 7 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (cargo test) |
| **Config file** | none |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo test -- --ignored` |
| **Estimated runtime** | ~3 seconds (unit) + ~2 seconds (PTY ignored) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo test -- --ignored`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 07-01-01 | 01 | 0 | IO-06 | integration (PTY) | `cargo test -- --ignored no_alternate_screen` | ❌ W0 | ⬜ pending |
| 07-01-02 | 01 | 0 | — | dev-dep | `cargo test` (compile check) | ❌ W0 | ⬜ pending |
| 07-01-03 | 01 | 1 | IO-06 | unit (existing) | `cargo test` | ✅ | ⬜ pending |
| 07-01-04 | 01 | 1 | IO-07 | unit (existing) | `cargo test terminal_guard` | ✅ | ⬜ pending |
| 07-01-05 | 01 | 1 | Regression | unit (all 94) | `cargo test` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/terminal_integration.rs` — PTY test asserting no `\x1b[?1049h` / `\x1b[?1049l`
- [ ] `Cargo.toml` — add `portable-pty = "0.9.0"` to `[dev-dependencies]`

*Existing infrastructure covers unit-level phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Terminal cursor visible after exit | IO-07 (cleanup) | Cursor visibility requires visual inspection | Run `gitmedit <file>`, press Esc, confirm blinking cursor in shell |
| Clean shell prompt on exit | IO-06 (D-02) | Visual "no artifacts" requires human eye | Run `gitmedit <file>`, press Esc, confirm clean prompt line with no TUI borders |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
