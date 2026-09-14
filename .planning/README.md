# .planning/ — frozen archive

**This directory is no longer the source of truth.** As of 2026-09-02 it is a read-only archive.
Planning for gitmedit lives in `openspec/`.

Nothing in here is maintained. Files below this line describe the project as it was understood on the
dates they carry, and several of their status fields were already wrong when this archive was frozen
(see *Corrections* below). Read them as history, not as current state.

## Where things went

| Was here | Is now |
|----------|--------|
| `PROJECT.md` — stack, constraints, conventions | `openspec/config.yaml`, `context:` |
| `REQUIREMENTS.md`, `milestones/v1.0-REQUIREMENTS.md` | `openspec/specs/<capability>/spec.md`, as requirements with scenarios |
| `ROADMAP.md` phases 1-6, 8 (shipped) | `openspec/changes/archive/2026-09-02-0*` |
| `ROADMAP.md` phase 7 (reverted) | `openspec/changes/restore-inline-rendering/` — active, because it is not actually done |
| `ROADMAP.md` phases 9-13 (pending) | `openspec/changes/{nano-chrome,rebase-reorder-merge-toolbar,standalone-commit,cross-platform-verification,colorize-comment-sections}/` |

Requirement ids (IO-*, CTX-*, EDIT-*, …) are cited in the migrated proposals, so any requirement can
be traced from here into the capability that now holds it.

## What stayed here, with no OpenSpec equivalent

OpenSpec models what the system does and what is changing. It has no home for process history, so all
of this remains here and nowhere else:

- `RETROSPECTIVE.md`, `STATE.md` — velocity metrics, session continuity, accumulated decisions
- `research/` — stack, architecture, features, and pitfalls research from before v1.0
- `debug/` — the rebase text-wrapping debugging session
- `todos/pending/` — two unresolved notes from 2026-03
- `milestones/` — the v1.0 roadmap, requirements, audit, and all v1.0 phase artifacts
- `phases/*/` — plans, summaries, verifications, discussion logs, patterns
- v1.2 backlog: `CONFIG-01`, `CONFIG-02`, `CONFIG-03` (configurable key bindings). Not built and not
  proposed, so it has no OpenSpec representation yet. Raise it as a change when it becomes real.

## Corrections made during migration

Requirement statuses here were verified against the code before being written into specs. Several
did not survive:

- `IO-06` (no alternate screen) and `IO-07` (panic-free terminal teardown) are marked complete here.
  They are not: commit `8dd89f1` reverted the Phase 7 fix, and `src/terminal.rs` today enters the
  alternate screen and calls `.unwrap()` in its `Drop`. Tracked as `restore-inline-rendering`.
- `EDIT-10` and `EDIT-11` are marked pending here. They shipped — Phase 8 is implemented and verified.
- `EDIT-03` (Ctrl+U) and `EDIT-07` (undo/redo) are marked pending here and are implemented.
- `INSTALL-05` (both editor configs respected) is marked pending here and is satisfied.
- `EDIT-06` claims text wraps at terminal width. It does not; the renderer scrolls horizontally and
  truncates. The spec records the actual behavior.
- `MERGE-02` (conflict markers not editable) was inverted by Phase 8; markers are editable now.
- `COMMIT-01`, `COMMIT-02`, `COMMIT-03` (subject counter, colors, blank-line warning) were removed by
  Phase 8. The capability was retired rather than carried forward.
- `PERF-02` and `PERF-03` have no test or instrumentation in the tree and could not be verified;
  they were left out of the specs rather than asserted.
- `PERF-01` was re-measured during migration: ~19ms per invocation over 50 runs, well inside budget.

The full matrix, including the evidence for each verdict, is summarized in the migrated proposals.
