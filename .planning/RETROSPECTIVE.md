# Retrospective

## Milestone: v1.0 — MVP

**Shipped:** 2026-04-07
**Phases:** 6 | **Plans:** 14

### What Was Built

- Git editor contract with RAII terminal safety and panic hook chaining
- Document model with ContentLine enum, comment/conflict marker parsing, round-trip serialization
- Full text editing with nano-style shortcuts and system clipboard
- Subject line counter with 50/72 color coding and blank line enforcement
- Context-aware help overlay (Ctrl+H)
- Structured rebase table with action cycling and color-coded rows
- Squash mode with protected commit log header
- Word-wrapped rebase subjects with variable-height scroll offset

### What Worked

- Phase-based delivery with clear success criteria kept scope tight
- TDD approach for data models (Document, RebaseLine parsing) caught edge cases early
- Single-crate structure avoided premature abstraction
- ratatui's widget model mapped well to the multi-mode UI

### What Was Inefficient

- Phase 2 plans still show as "Not started" in ROADMAP despite having summaries — roadmap update didn't fire correctly
- IO-06 regression went undetected until milestone audit — no continuous regression testing across phases
- Some SUMMARY.md one-liner fields were left empty, reducing archival quality

### Patterns Established

- Drop-based RAII for terminal cleanup (TerminalGuard pattern)
- Atomic file writes via rename for git file safety
- Three-branch event loop: help_visible → rebase_mode → normal_editing
- ContentLine enum as the editing/rendering boundary between git content and user text

### Key Lessons

- Milestone audit should run after each phase, not just at the end — IO-06 regression would have been caught earlier
- Verification coverage matters — Phases 2 and 5 had no VERIFICATION.md, leading to audit gaps
- Keep SUMMARY.md one-liners populated — they feed directly into milestone records

## Cross-Milestone Trends

| Metric | v1.0 |
|--------|------|
| Phases | 6 |
| Plans | 14 |
| Tasks | 24 |
| LOC | 2,448 |
| Tests | 94 |
| Timeline | 15 days |
| Audit score | 41/50 |
