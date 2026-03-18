# Phase 1: Git Contract + TUI Shell - Context

**Gathered:** 2026-03-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Binary reads a file from argv[1], displays it in a TUI that does not use alternate screen, exits with code 0 on save or code 1 on cancel — with terminal always safely restored even if the process panics.

</domain>

<decisions>
## Implementation Decisions

### Terminal Safety
- Use Drop guard + panic hook pattern for raw mode recovery
- Panic hook ensures terminal is restored even if the guard doesn't run
- Based on crossterm design guarantees documented in official examples

### File I/O Strategy
- Write-to-temp-then-rename (atomic write) for safety
- Prevents corruption if editor crashes mid-write
- Standard approach for all file editors

### Context Detection
- Match filename only (COMMIT_EDITMSG, MERGE_MSG, git-rebase-todo, SQUASH_MSG)
- Simple and matches git's own detection logic
- No need for full path matching at this phase

### Dependency Minimalism
- Keep core dependencies minimal: ratatui, crossterm, anyhow, thiserror, clap
- No async runtime (synchronous event polling — editor is invoked synchronously by git)
- No serde, tokio, regex — avoid bloat
- No alternate screen (render into main buffer like nano)

### Exit Code Contract
- Exit 0 on successful save
- Exit 1 on cancel or error
- This contract is non-negotiable — it determines whether git proceeds or aborts

### Startup Performance
- Target: <100ms on typical hardware (nano baseline ~50ms)
- Minimal feature flags in dependencies
- Pre-allocate buffers where reasonable
- Measure with `time gitmedit <file>` during Phase 1

### Claude's Discretion
- Exact panic hook implementation details
- Temp file location and naming strategy
- Cursor rendering during initial display
- Terminal size detection and handling

</decisions>

<canonical_refs>
## Canonical References

### Architecture & Design
- `.planning/research/ARCHITECTURE.md` — Component structure, layer separation, data flow
- `.planning/research/STACK.md` — Technology choices with rationale, versions (ratatui 0.30, crossterm 0.29)
- `.planning/ROADMAP.md` Phase 1 — Phase goal, success criteria, requirements

### Requirements
- `.planning/REQUIREMENTS.md` IO-01 through IO-06 — File I/O requirements
- `.planning/REQUIREMENTS.md` CTX-01, CTX-02 — Context detection (Phase 1 subset)
- `.planning/REQUIREMENTS.md` PERF-01 — Startup performance target

### Key Pitfalls
- `.planning/research/PITFALLS.md` — Terminal raw mode handling, exit code contract, panic safety (critical for Phase 1)

</canonical_refs>

<code_context>
## Existing Code Insights

### Starting State
- Cargo.toml exists with minimal config (edition 2024)
- No src/ files yet — greenfield Rust project
- Monorepo structure ready: `crates/gitmedit/`

### Research-Provided Architecture
- ARCHITECTURE.md defines 8 modules: main, context, parser, document, app, input, renderer, writer
- Elm-style single App state owner pattern (no shared state between input/renderer)
- Clear separation of concerns: detection, parsing, editing, rendering, writing

### Integration Points
- Entry: `main.rs` parses argv[1], wires all layers
- Exit: `main.rs` writes file and returns exit code to git
- No external integration needed for Phase 1 (git integration is just argv + exit code)

</code_context>

<specifics>
## Specific Ideas

- Terminal behavior should match nano exactly: display in main buffer, no alternate screen, fast startup
- Git integration is minimal: just argv[1] → file path, exit code → git decision
- Panic safety is critical first architecture decision — get this right before building UI

</specifics>

<deferred>
## Deferred Ideas

- Comment line rendering styles — Phase 2 (when we parse comments)
- Hotkey help display — Phase 3 (commit message intelligence)
- Rebase/squash modes — Phase 4
- Installation & distribution — Phase 5

</deferred>

---

*Phase: 01-git-contract-tui-shell*
*Context gathered: 2026-03-18*
