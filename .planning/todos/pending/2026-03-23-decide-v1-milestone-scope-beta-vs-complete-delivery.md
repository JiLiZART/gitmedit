---
created: 2026-03-23T20:09:39.406Z
title: Decide v1.0 milestone scope — beta vs complete delivery
area: planning
files:
  - .planning/ROADMAP.md
  - .planning/REQUIREMENTS.md
  - .planning/STATE.md
---

## Problem

Current implementation status:
- ✅ Phase 1: Git Contract + TUI Shell (complete)
- ⏳ Phase 2: Text Editing + Comment Handling (0/3 plans, but most requirements marked complete)
- ✅ Phase 3: Commit Message Intelligence (complete)
- ✅ Phase 4: Rebase + Squash Modes (complete)
- ⏳ Phase 5: Installation + Distribution (not started)

REQUIREMENTS.md shows most v1 requirements are satisfied despite Phase 2 not being formally planned/executed. Phase 2 requirements marked "Complete" even though the phase is at 0/3 plans.

Need to decide: Ship as **v0.1-beta** (phases 1,3,4 + partial phase 2 features) or **v1.0** (complete all 5 phases first including full text editing and installation)?

## Solution

Options to evaluate:
1. **Ship v0.1-beta** — Archive phases 1,3,4 as early release; continue with 2,5 in v1.0
2. **Audit Phase 2 implementation** — Check if requirements actually satisfied despite plan structure; if so, consolidate into v1.0
3. **Complete phases 2 & 5 first** — Finish text editing (Ctrl+U, undo/redo) and installation before shipping v1.0

TBD: User decision on which path balances shipping velocity vs completeness.
