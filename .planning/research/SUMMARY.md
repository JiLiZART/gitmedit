# Project Research Summary

**Project:** gitmedit
**Domain:** Rust TUI git commit/rebase editor (`core.editor` and `sequence.editor` replacement)
**Researched:** 2026-03-18
**Confidence:** HIGH

## Executive Summary

gitmedit is a narrow-scope, single-binary Rust TUI tool that git invokes as a blocking subprocess to edit commit messages and rebase todo files. The implementation domain is well-understood: the Rust TUI ecosystem has a clear, stable stack (ratatui 0.30 + crossterm 0.29 + ratatui-textarea 0.8), the git editor contract is formally documented, and prior art (git-interactive-rebase-tool, lazygit) defines the feature ceiling. There is no ambiguity about what this product must do — the contracts are git's, not ours. The primary engineering challenges are correctness, not novelty.

The recommended approach is to build around ratatui's immediate-mode rendering loop with a strict Elm-inspired separation: a typed `Document` model, an `Action` enum as the event bus, a read-only `Renderer`, and a `FileWriter` that is called only once on save. File I/O happens entirely outside the event loop. Git context (Commit / Merge / Rebase / Squash) is detected from the filename passed as `argv[1]` and drives all conditional rendering. The binary is installed once and configured via both `git config core.editor gitmedit` and `git config sequence.editor gitmedit`.

The top risks are all correctness-class failures: leaving the terminal in raw mode after a panic, returning exit code 0 on cancel (which silently proceeds a git operation), corrupting a rebase todo file on write-back, and hardcoding `#` as the comment character without reading `core.commentChar`. Every one of these has shipped as bugs in mature tools (GitExtensions, Atom, VSCode, Magit). They must be addressed in Phase 1 — they are cheap to prevent and expensive to retrofit. The secondary risk is the alternate screen decision: using ratatui's default `EnterAlternateScreen` hides the user's prior `git diff` output, which is a significant UX regression versus nano. This architectural decision must be made in Phase 1 because changing it later requires restructuring all terminal init/restore code.

## Key Findings

### Recommended Stack

The stack is minimal and well-justified. ratatui 0.30 (the community continuation of the archived tui-rs) is the de-facto standard for Rust TUIs with 11.9M+ downloads. ratatui-textarea 0.8 (maintained by the official ratatui org) provides multi-line editing, undo/redo, cursor highlighting, and crossterm event passthrough — eliminating roughly two weeks of text buffer implementation work. crossterm 0.29 handles raw mode, alternate screen, and cross-platform keyboard events. Error handling uses the idiomatic anyhow + thiserror pairing. clap 4.6 handles the single positional argument with `--help` and `--version` for free. No async runtime (tokio) is needed or wanted: git invokes the editor synchronously, and adding async overhead is unjustified for a single-file processor.

**Core technologies:**
- ratatui 0.30: TUI rendering framework — de-facto standard, immediate-mode, sub-millisecond frames
- crossterm 0.29: Terminal backend — cross-platform raw mode and keyboard events, only valid choice for cross-platform
- ratatui-textarea 0.8: Multi-line editor widget — official ratatui org, eliminates custom text buffer implementation
- anyhow 1.0 + thiserror 2.0: Error handling — ergonomic propagation with typed domain errors
- clap 4.6: CLI argument parsing — zero-boilerplate positional arg with future-flag extensibility

**Critical version note:** ratatui 0.30 ships a modular workspace; `ratatui-textarea 0.8` pins to `ratatui-core ^0.1` from that workspace. Pin both together to avoid resolver conflicts.

See `.planning/research/STACK.md` for alternatives considered and version compatibility table.

### Expected Features

gitmedit operates in a niche with no direct competitors — no existing tool serves as a dedicated git-aware TUI for both `core.editor` and `sequence.editor` roles. The feature set is small but the correctness bar is high.

**Must have (table stakes):**
- Open file from `argv[1]` and display in editable buffer — the entire premise of the tool
- Basic text editing (type, delete, navigate, newlines) — fundamental
- Save with Ctrl+S (exit 0) and cancel with Esc (exit non-zero) — git editor contract
- Comment lines (`#`) rendered visually distinct and preserved verbatim on save — prevents confusion and file corruption
- Visible keybinding bar at bottom — table stakes since nano popularized it; required for discoverability
- Handle COMMIT_EDITMSG and MERGE_MSG — the two most common invocation paths
- Handle git-rebase-todo format — required for interactive rebase to function
- Correct exit codes (0 = save, non-zero = cancel) — correctness; wrong codes risk data loss

**Should have (competitive differentiators for v1.x):**
- Subject line character counter with 50/72 color coding — high daily utility, low implementation cost
- Rebase action cycling with hotkeys (p/s/f/d/r) — the natural next step once rebase-todo display works
- Squash context rendering (accumulated commit log as styled read-only block) — reduces confusion during squash merges
- Line-length ruler at column 72 in commit body — low-cost quality-of-life feature
- Git context label in status bar (COMMIT / MERGE / REBASE) — confirms to the user what they are editing

**Defer to v2+:**
- Rebase-todo line reordering — complex interaction model; validate basic action manipulation first
- Undo/redo history — basic editing without undo is acceptable for short commit sessions
- Mouse support — fragile across terminal emulators; keyboard-only is the right default

**Deliberate anti-features:** No vim/emacs keybindings, no plugin system, no config file, no syntax highlighting of commit body text, no spell checking.

See `.planning/research/FEATURES.md` for dependency graph and full prioritization matrix.

### Architecture Approach

The architecture follows ratatui's standard Elm-inspired pattern with strict component separation. All state lives in an `App` struct that owns a `Document` (typed text model with editable lines and comment regions as distinct collections). An `InputHandler` converts raw crossterm `KeyEvent`s into a typed `Action` enum; `App::apply(action)` mutates state and returns an `Outcome`; a read-only `Renderer` paints widgets from `App` each frame. File I/O is bookended outside the event loop: parse at startup, write once on save. The `GitContext` enum (Commit / Merge / Rebase / Squash) is detected from the filename at startup and passed to both the parser (to classify lines) and the renderer (to select the correct UI layout).

**Major components:**
1. `context.rs` + `FileDetector` — infer `GitContext` from `argv[1]` filename; no content inspection needed for type detection
2. `document.rs` + `Document` — in-memory text model: `editable_lines: Vec<String>`, `comment_lines: Vec<(usize, String)>`, `cursor: (row, col)`; fully unit-testable without a terminal
3. `parser.rs` + `FileParser` — raw bytes to `Document`; splits editable vs. comment content based on `GitContext` and `core.commentChar`
4. `app.rs` + `App` — single state owner; applies `Action` enum; returns `Outcome::Save | Cancel | Continue`
5. `input.rs` + `InputHandler` — `KeyEvent` to `Action`; testable by constructing `Action` values directly
6. `renderer.rs` + `Renderer` — reads `&App` read-only; calls `terminal.draw()` per frame; context-aware layout selection
7. `writer.rs` + `FileWriter` — serializes `Document` to disk in original line order; preserves comment lines byte-for-byte; called once on save
8. `main.rs` — wires all layers; owns event loop; maps `Outcome` to `process::exit(0|1)`

**Build order:** document → context + parser → writer → app → input → renderer → main. Each layer is testable without a terminal except `renderer` (use ratatui's `TestBackend`).

See `.planning/research/ARCHITECTURE.md` for full data flow diagrams and anti-patterns.

### Critical Pitfalls

All six critical pitfalls identified in research have shipped as bugs in mature tools. They map to Phase 1 and Phase 2 prevention requirements.

1. **Terminal not restored after panic** — install a panic hook before any terminal init that calls `disable_raw_mode()` and `LeaveAlternateScreen`; also implement `Drop`-based cleanup; do not rely on only one mechanism. (Phase 1)

2. **Wrong exit code on cancel** — map Cancel/Esc to `process::exit(1)`, Save to `process::exit(0)`; never call `exit(0)` unconditionally; verify with a real `git commit` session. (Phase 1)

3. **Alternate screen hides prior `git diff` output** — do NOT use `EnterAlternateScreen`; render into the main terminal buffer like nano; this is a foundational Phase 1 architecture decision. (Phase 1)

4. **Hardcoded `#` as comment delimiter** — read `core.commentChar` via `git config --get core.commentChar` at startup; fall back to `#` only if absent; handle `auto` value. (Phase 1, when comment display is implemented)

5. **Rebase todo file corrupted on write-back** — write atomically (temp file + rename); preserve all lines byte-for-byte unless user explicitly edited them; never reconstruct todo lines from parsed structs; test with real `git rebase -i`. (Phase 2)

6. **`core.editor` vs `sequence.editor` split** — installation docs must set both configs; code must handle both COMMIT_EDITMSG and git-rebase-todo invocations; detect context from filename. (Phase 2, before any rebase feature work)

See `.planning/research/PITFALLS.md` for full warning signs, recovery costs, and the "Looks Done But Isn't" checklist.

## Implications for Roadmap

Based on research, the dependency graph is clear and dictates phase order. The git editor contract is the load-bearing foundation; multi-context file handling depends on a working contract; differentiator features depend on multi-context handling.

### Phase 1: Core TUI Skeleton and Git Contract

**Rationale:** The panic hook, exit code contract, alternate screen decision, and `core.commentChar` support are all Phase 1 concerns per PITFALLS.md. These are cheap to do first and expensive to retrofit. The architecture build order (document → parser → app → renderer) establishes the component boundaries that all later phases extend. Without a correct git editor contract, nothing else ships.

**Delivers:** A working git editor for standard commits — open COMMIT_EDITMSG, edit, save (exit 0) or cancel (exit non-zero), comment lines preserved and styled. Fast startup. Panic-safe terminal restore. No alternate screen.

**Addresses from FEATURES.md (P1):** File open from argv, basic text editing, Save/Cancel with correct exit codes, comment line styling, keybinding bar, COMMIT_EDITMSG + MERGE_MSG handling.

**Avoids from PITFALLS.md:** Terminal raw mode on panic, wrong cancel exit code, alternate screen UX regression, hardcoded comment char.

**Stack:** ratatui 0.30, crossterm 0.29, ratatui-textarea 0.8, clap 4.6, anyhow + thiserror. Full Cargo.toml from STACK.md.

**Architecture:** All 8 components established in their final form. document.rs and writer.rs fully covered by unit tests.

### Phase 2: Multi-Context File Handling and Rebase Support

**Rationale:** The `core.editor` / `sequence.editor` split (Pitfall 3) must be validated before any rebase feature work. Once the binary handles git-rebase-todo correctly, the `GitContext` enum drives different UI layouts via the already-built `Renderer`. The `FileDetector` + `FileParser` components already exist from Phase 1 — this phase extends them for rebase-todo format parsing and structured action display.

**Delivers:** Full rebase-todo display with structured action rows. `gitmedit` works correctly as both `core.editor` and `sequence.editor`. Context label in status bar. Correct atomic write-back of rebase-todo files.

**Addresses from FEATURES.md (P1 remaining):** Rebase-todo file handling, git context detection, sequence editor role.

**Avoids from PITFALLS.md:** `core.editor` vs `sequence.editor` confusion, rebase todo file corruption on write-back, no context display for user orientation.

**Architecture extension:** `FileDetector` extended for git-rebase-todo; `FileParser` adds structured todo-line parsing; `Renderer` adds todo-list layout mode.

### Phase 3: Git-Aware Differentiator Features

**Rationale:** These features depend on a fully working Phase 2 (context detection must be solid before subject-line counter or rebase action hotkeys are layered on). They are all P2 priority from FEATURES.md — high user value, medium-or-low implementation cost. Building them together makes sense because they all read from `GitContext` and extend `Renderer` styling.

**Delivers:** Subject line character counter (50/72 color coding), rebase action cycling via hotkeys (p/s/f/d/r keys), squash context rendering (accumulated commit log as styled read-only block), line-length ruler at column 72.

**Addresses from FEATURES.md (P2):** All four v1.x differentiator features from the prioritization matrix.

**Avoids from PITFALLS.md:** Rebase action manipulation conflicts with free-form editing — use the hotkey-cycling approach (Pitfall note in FEATURES.md dependency section) rather than allowing direct text editing of action keywords.

### Phase 4: Polish, Distribution, and Edge Cases

**Rationale:** Once core functionality is validated via dogfooding, final polish addresses the "Looks Done But Isn't" checklist from PITFALLS.md and prepares for public distribution. Binary size optimization (release profile tuning already specified in STACK.md) and installation documentation (both `core.editor` and `sequence.editor` instructions) belong here.

**Delivers:** Empty-message save warning with confirmation prompt, Esc-with-changes confirmation prompt, terminal resize handling, `--version` / `--help` output, `cargo install gitmedit` distribution path, full PITFALLS.md verification checklist completed.

**Addresses from FEATURES.md:** UX polish for Esc confirmation; discoverability for rebase action key legend.

**Avoids from PITFALLS.md:** Empty message saves without warning (UX pitfall), fat-finger Esc discards unsaved changes without confirmation.

### Phase Ordering Rationale

- **Correctness before features:** The git editor contract (exit codes, file preservation, terminal restore) must be correct before any differentiator features are built. A commit editor that silently discards work is worse than no editor.
- **Architecture dependency order:** ARCHITECTURE.md's build order (document → parser → app → renderer) maps directly to Phase 1's internal ordering. Each component is testable in isolation before the next is added.
- **Context detection as multiplier:** FEATURES.md explicitly identifies git context detection as the feature that unlocks subject-line counter, rebase actions, and squash rendering. Build it in Phase 2; extend it in Phase 3.
- **Pitfall prevention is front-loaded:** Five of the six critical pitfalls from PITFALLS.md are Phase 1 concerns. The one Phase 2 pitfall (rebase todo corruption) is addressed as the first task of Phase 2, before any rebase feature work begins.

### Research Flags

Phases likely needing deeper research during planning:

- **Phase 2:** `core.commentChar = auto` handling requires scanning the file to find a safe delimiter character — the algorithm is not fully specified in research and may need a concrete implementation plan before coding begins.
- **Phase 2:** Atomic write on Windows (temp file + rename in `.git/`) has different behavior than on Unix; if Windows support is in scope, this needs verification.
- **Phase 3:** Squash context detection (identifying the "This is a combination of N commits" comment block in COMMIT_EDITMSG) requires reading the exact git-generated comment format — verify against git source or docs during planning.

Phases with standard patterns (no additional research needed):

- **Phase 1:** ratatui's immediate-mode loop, panic hook setup, and crossterm raw mode handling are fully documented in official ratatui.rs docs and the STACK.md / PITFALLS.md sources. Established patterns — implement directly.
- **Phase 4:** Binary release profile tuning and `cargo install` distribution are fully specified in STACK.md. No research needed.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All versions verified against crates.io; compatibility matrix confirmed; ratatui 0.30 + ratatui-textarea 0.8 dependency resolution explicitly checked |
| Features | HIGH (table stakes) / MEDIUM (differentiators) | Table stakes derived from official git docs; differentiator features have fewer prior art examples in this narrow niche |
| Architecture | HIGH | Patterns verified against official ratatui.rs docs; Elm-style separation is the canonical ratatui architecture; component boundaries derived from build-order analysis |
| Pitfalls | HIGH (git contract) / MEDIUM (TUI-specific) | Git exit code and commentChar pitfalls backed by real bug reports in mature tools (GitExtensions, Atom, VSCode, Magit); TUI raw-mode pitfall confirmed by crossterm maintainer |

**Overall confidence:** HIGH

### Gaps to Address

- **`core.commentChar = auto`:** The algorithm for selecting a safe character when `auto` is configured is not documented in the research sources. During Phase 2 planning, consult git source (`commit.c`) or test empirically to determine the selection logic.
- **Windows terminal compatibility:** crossterm handles Windows in principle; actual behavior with Windows Terminal vs. cmd.exe vs. ConEmu for raw mode, alternate screen suppression, and `KeyEventKind::Press` filtering needs hands-on verification if Windows support is a stated goal.
- **ratatui-textarea vs. hand-rolled buffer for rebase-todo:** ratatui-textarea is recommended for commit message editing. For rebase-todo structured row editing (where individual cells have constrained values), it may be simpler to render the todo list as custom widgets rather than a textarea. Evaluate during Phase 2 planning.
- **Undo/redo scope for v1:** ratatui-textarea provides undo/redo built-in. The decision to defer undo/redo to v2 may be revisited during Phase 1 since it comes for free from the widget — the deferred cost may be zero.

## Sources

### Primary (HIGH confidence)
- [crates.io — ratatui 0.30.0](https://crates.io/crates/ratatui) — version and workspace structure confirmed
- [crates.io — ratatui-textarea 0.8.0](https://crates.io/crates/ratatui-textarea) — version, ratatui org ownership confirmed
- [crates.io — crossterm 0.29.0](https://crates.io/crates/crossterm) — version confirmed
- [ratatui.rs — Application Patterns](https://ratatui.rs/concepts/application-patterns/) — Elm architecture, component patterns, Action-based communication
- [ratatui.rs — Setup Panic Hooks](https://ratatui.rs/recipes/apps/panic-hooks/) — panic hook implementation
- [Git commit documentation](https://git-scm.com/docs/git-commit) — COMMIT_EDITMSG behavior, exit code contract
- [Git rebase documentation](https://git-scm.com/docs/git-rebase) — rebase-todo format, sequence.editor config
- [crossterm Issue #368](https://github.com/crossterm-rs/crossterm/issues/368) — raw mode not restored after panic, maintainer confirmed
- [GitExtensions Issue #3560](https://github.com/gitextensions/gitextensions/issues/3560) — core.commentChar bug in mature tool
- [Atom GitHub PR #1988](https://github.com/atom/github/pull/1988) — core.commentChar fix as prior art
- [MitMaro/git-interactive-rebase-tool](https://github.com/MitMaro/git-interactive-rebase-tool) — reference implementation for rebase sequence editor

### Secondary (MEDIUM confidence)
- [Baeldung: Configure Core and Sequence Git Editors](https://www.baeldung.com/ops/git-editors-select-configure) — core.editor vs sequence.editor split; verified against git 2.40 release notes
- [cbea.ms: How to Write a Git Commit Message](https://cbea.ms/git-commit/) — 50/72 rule; widely cited convention
- [HN: Show HN: I made a git rebase TUI editor](https://news.ycombinator.com/item?id=41831073) — community expectations for rebase TUI editors

### Tertiary (LOW confidence)
- WebSearch patterns for git editor startup latency expectations — consistent across multiple sources but not formally benchmarked

---
*Research completed: 2026-03-18*
*Ready for roadmap: yes*
