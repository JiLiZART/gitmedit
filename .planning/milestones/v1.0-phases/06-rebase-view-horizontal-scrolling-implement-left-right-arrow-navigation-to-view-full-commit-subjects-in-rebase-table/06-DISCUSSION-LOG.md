# Phase 6: Rebase View — Full Commit Subject Visibility - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-28
**Phase:** 06-rebase-view-horizontal-scrolling
**Areas discussed:** Scroll scope, Wrap style

---

## Scroll Scope (pivoted to wrapping)

| Option | Description | Selected |
|--------|-------------|----------|
| Scroll just subject column | Only the subject column scrolls horizontally | |
| Scroll entire row | The whole row scrolls left/right | |
| Wrap to next line | No scrolling — wrap long subjects to continuation lines | ✓ |

**User's choice:** "Don't scroll any content horizontally, just wrap it to next line"
**Notes:** User rejected the entire horizontal scrolling premise. Wrapping is the desired approach.

---

## Wrap Style

| Option | Description | Selected |
|--------|-------------|----------|
| Indent under subject | Continuation lines align under the subject column (past action+hash) | ✓ |
| Full-width wrap | Continuation flows from the left edge, no column alignment | |
| You decide | Claude picks whatever looks best | |

**User's choice:** Indent under subject
**Notes:** Keeps the table readable with clear column structure

---

## Claude's Discretion

- Exact ratatui mechanism for multi-line row wrapping
- Visual cues for wrapped vs non-wrapped rows
- Scroll offset calculation adjustments for variable-height rows

## Deferred Ideas

None
