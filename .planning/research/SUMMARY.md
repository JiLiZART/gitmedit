# Project Research Summary

**Project:** gitmedit v1.1 — Standalone Commit + Editor Overhaul
**Domain:** TUI git editor (Rust, ratatui + crossterm) — incremental feature addition to working v1.0 codebase
**Researched:** 2026-04-07
**Confidence:** HIGH

## Executive Summary

gitmedit v1.1 is a feature-addition milestone on top of a validated, shipped Rust TUI application. The existing stack (ratatui 0.30, crossterm 0.29, ratatui-textarea 0.8, clap 4.6, arboard 3.6) is stable and must not be upgraded. The new features divide cleanly into two tracks: rendering improvements (nano-style chrome, merge toolbar) and behavioral additions (standalone commit mode, rebase line reordering, exec line editing, custom hotkey configuration). All new capabilities can be built without heavy new dependencies — only `tempfile` (promoted from dev-dep), `toml`, `serde`, `dirs`, and `crokey` are added, keeping the binary well under 1MB.

The single most important architectural finding is that the 9 new features have a strict dependency ordering that must be respected. IO-06 (alternate screen regression) must be fixed first because it corrupts the evaluation of all rendering changes. Nano-style chrome must come before the merge toolbar because the toolbar needs the chrome footer to exist. Custom hotkey configuration must come last because it cross-cuts every input handler and requires all other features to be finalized. Three of the nine features (rebase reordering, exec line editing, standalone commit mode) are architecturally independent of the rendering track and can be developed sequentially after the plain editor default is landed.

The dominant risk cluster is process-lifecycle correctness in standalone commit mode: the terminal guard's `Drop` implementation contains a `.unwrap()` that will cause a double-panic if the subprocess fails while the TUI is tearing down. This is pre-existing code debt that must be fixed before standalone commit mode is implemented. Secondary risks include `selectable_indices` desync during rebase reordering (data loss / wrong serialized order) and the "plain editor" refactor silently breaking squash mode if `Document` internals are changed incorrectly. Both have clear preventions documented in the architecture research: regenerate `selectable_indices` after every reorder, and implement plain-editor mode as a flag on `Document` rather than changing the classification logic globally.

## Key Findings

### Recommended Stack

The v1.0 stack is locked and validated — do not upgrade any existing dependency. The v1.1 additions are minimal by design: `tempfile` moves from dev-dependency to runtime (no version change), `toml 0.8` + `serde 1.0` + `dirs 6.0` cover config file loading, and `crokey 1.4` (with its `serde` feature) converts human-readable key strings from TOML into crossterm `KeyEvent` values. For testing, `insta = "1"` is added as a dev-dependency alongside ratatui's existing `TestBackend`. No git library (`git2`, `gix`) is needed — a single `std::process::Command` handles `git commit -F`. No async runtime is needed. No regex crate is needed for MERGE_MSG parsing (stdlib `str::starts_with` and `str::trim_start_matches` suffice).

**Core technologies (v1.1 additions only):**
- `tempfile 3` (runtime): Named temp file for standalone commit message — already in lockfile as dev-dep, promote with no version change
- `toml 0.8`: Config file parsing — 0.8.x stable series; 0.9 introduced breaking API changes with no benefit for this use case
- `serde 1.0` (with derive): Config struct deserialization — already an indirect dep, adding explicitly costs nothing new at compile time
- `dirs 6.0`: Platform-correct config directory resolution (`~/.config` on Linux, `~/Library/Application Support` on macOS, `%APPDATA%` on Windows)
- `crokey 1.4` (with serde): Human-readable key string to crossterm `KeyCombination` — production-proven in `broot`; confirm version with `cargo add crokey@1.4`
- `insta 1` (dev-dep): Snapshot assertions on `TestBackend` frames — official ratatui testing pattern documented at ratatui.rs

### Expected Features

All v1.0 table-stakes features are shipped. The v1.1 must-ship set is 7 features; 1 is conditional on complexity; 2 are deferred.

**Must have (table stakes for v1.1):**
- IO-06 alternate screen fix — pre-existing regression; blocks correct evaluation of all rendering changes
- Plain editor default — remove confusing read-only behavior on comment lines in non-rebase contexts
- Nano-style chrome (header bar + bottom command bar) — modeled on GNU nano; prerequisite for merge toolbar
- Standalone commit mode — headline feature; `gitmedit` with no args opens TUI, then invokes `git commit -F <tmpfile>`
- Merge commit toolbar — parse `# Conflicts:` block from MERGE_MSG; display conflict count and file list in footer
- Rebase line reordering — table stakes for any interactive rebase TUI; Alt+Up / Alt+Down (Shift variants as alternative)
- Exec line argument editing — completes rebase feature set; modal overlay edits the `subject` field of exec lines

**Should have (ships if complexity allows):**
- Custom hotkey configuration — TOML config at `~/.config/gitmedit/config.toml`; must be implemented last due to cross-cutting impact

**Defer to v2+:**
- Cross-platform testing (Windows Terminal) — testing effort, not implementation; do as a followup after features ship
- Undo/redo (EDIT-07) — low priority for short-session commit editor; adds state management complexity

**Anti-features (do not build):** Vim/Emacs keybindings, plugin system, auto-staging, spell check, conflict resolution UI, syntax highlighting of commit body text.

### Architecture Approach

gitmedit v1.0 has a clean 7-module architecture that all v1.1 features fit into without requiring new modules (except a possible `config.rs` for hotkeys). The renderer layout changes from 2-slot (`[Min(0), Length(1)]`) to 3-slot (`[Length(1), Min(0), Length(1)]`). The most significant new data flow is `InvocationMode` (git-managed vs standalone) in `main.rs`, and `merge_metadata: Option<MergeMetadata>` computed once at `App::new()` and consumed at render time only — never re-parsed per frame.

**Major components and their v1.1 changes:**
1. `main.rs` — MODIFY: `Cli.path` becomes `Option<PathBuf>`; new `InvocationMode` enum; standalone git commit flow; Shift+Up/Down and Enter-on-exec key bindings
2. `app.rs` — MODIFY: `display_path: String`, `merge_metadata: Option<MergeMetadata>`, `exec_edit_mode` + `exec_edit_textarea` fields; new Action variants (`MoveRebaseLineUp/Down`, `EnterExecEdit`, `CommitExecEdit`, `CancelExecEdit`); dual-swap reorder logic (both `rebase_lines` and `selectable_indices`)
3. `document.rs` — MODIFY: `EditorMode` enum (`Plain` / `GitAware`); `MergeMetadata` struct; `parse_merge_metadata()` function
4. `renderer.rs` — MODIFY: 3-slot layout; new `render_header()`; consolidated `render_footer()` (replaces 3 separate status bar functions); new `render_exec_edit_overlay()`
5. `terminal.rs` — FIX: IO-06 alternate screen regression; `.unwrap()` in `Drop` replaced with `let _ = crossterm::execute!(...)`
6. `context.rs`, `writer.rs` — NO CHANGE
7. `config.rs` — NEW (only if custom hotkeys ship): load TOML at startup, merge with hardcoded defaults, produce key-action map

### Critical Pitfalls

1. **Terminal guard `.unwrap()` in `Drop` causes double-panic on subprocess failure** — Replace with `let _ = crossterm::execute!(...)` in `terminal.rs` before implementing standalone commit mode. Always call `drop(guard)` explicitly before invoking `git commit -F`. Fix this in Phase 1 unconditionally.

2. **`selectable_indices` desyncs after rebase reorder past a Comment line** — After every reorder swap, regenerate `selectable_indices` from scratch via O(n) scan of `rebase_lines`. Do not update it incrementally. Write a test with `pick A / # comment / pick B` and verify moving B up produces correct serialized output.

3. **Plain editor default silently corrupts squash/merge modes** — Implement `EditorMode::Plain` as a mode flag that changes `classify_line()` behavior; do not remove the `Comment`/`ConflictMarker` variants from `Document::parse` globally. Run all existing squash and merge roundtrip tests after every change to `Document`.

4. **Nano chrome layout breaks scroll calculations by passing wrong `area` to sub-renderers** — Define the 3-slot layout once in the top-level `Renderer::render()` and pass `chunks[1]` (content area) to all sub-renderers. A single missed path that passes the full frame produces incorrect `visible_height` and broken cursor scrolling.

5. **MERGE_MSG `# Conflicts:` format varies across git versions and locales** — Parse defensively. If the section is absent or unrecognized, show nothing. Test against git 2.39 and 2.47. Treat any missing section as "no conflict info" and hide the toolbar element rather than showing empty data.

## Implications for Roadmap

The dependency ordering is strict at the top and flexible in the middle. IO-06 must be first. Chrome must precede merge toolbar. Everything else has implementation flexibility. Suggested 5-phase structure:

### Phase 1: Foundations and Regression Fix
**Rationale:** IO-06 makes it impossible to correctly evaluate any rendering change — the terminal is in the wrong state for the nano model. The `.unwrap()` in `TerminalGuard::Drop` is a pre-existing time bomb that standalone commit mode will detonate. Both are surgical single-file fixes with near-zero regression risk. Landing them first gives a trustworthy baseline for everything else.
**Delivers:** Terminal renders in the main scroll buffer (no alternate screen); guard drop is panic-safe; existing test suite confirms zero regression
**Addresses:** IO-06 alternate screen fix (must-ship), `.unwrap()` Drop fix (critical pitfall prevention)
**Avoids:** Terminal stuck in raw mode after subprocess failure (Pitfall 1, 9)

### Phase 2: Editor Model Simplification
**Rationale:** Plain editor default touches `Document` internals. It is safest to land this before the renderer is restructured in Phase 3 to avoid concurrent changes to the same data path. The `EditorMode` enum it introduces also clarifies the model for all subsequent phases.
**Delivers:** All lines editable in non-rebase, non-squash contexts; `EditorMode` enum in `document.rs`; cargo test confirms squash/merge modes unaffected
**Addresses:** Plain editor default (must-ship)
**Avoids:** Squash/merge mode corruption from Document internals change (Pitfall 3)

### Phase 3: Nano Chrome
**Rationale:** Chrome is a hard prerequisite for the merge toolbar (Phase 4). Restructuring the renderer layout once here means all subsequent footer/header additions land cleanly on the 3-slot structure. Validating chrome in all 4 modes (commit, merge, rebase, squash) also serves as a regression test for Phase 2.
**Delivers:** Header bar (context label left, filename right), bottom command bar (context-sensitive key hints), consolidated `render_footer()`, 3-slot layout across all modes
**Uses:** ratatui `Layout`, `Block`, `Paragraph`, `Span` — no new dependencies (STACK.md)
**Implements:** `render_header()` + `render_footer()` in `renderer.rs`; `display_path: String` added to `App`
**Avoids:** Scroll calculation breakage from wrong area passed to sub-renderers (Pitfall 4) — layout defined once at top level

### Phase 4: Rebase Feature Completion + Merge Toolbar
**Rationale:** Rebase reordering and exec editing are independent of each other and of the rendering track, but both depend on the v1.0 rebase table being stable. Grouping them with merge toolbar keeps all "complete git workflow coverage" work in one phase. Merge toolbar specifically requires the chrome footer from Phase 3.
**Delivers:** Rebase lines moveable up/down; exec line commands editable via modal overlay; merge commit footer showing conflict count and affected file paths
**Uses:** `Vec::swap()` + `selectable_indices` regeneration; `MergeMetadata` struct; `parse_merge_metadata()` in `document.rs`; modal overlay pattern from existing `render_help_overlay()`
**Implements:** `MoveRebaseLineUp/Down` Actions in `app.rs`; `EnterExecEdit`/`CommitExecEdit`/`CancelExecEdit` lifecycle; `render_exec_edit_overlay()` in `renderer.rs`; `parse_merge_metadata()` in `document.rs`
**Avoids:** `selectable_indices` desync (Pitfall 5); exec hash/subject field confusion (Pitfall 6); MERGE_MSG format variance (Pitfall 7)

### Phase 5: Standalone Commit Mode + (Optional) Custom Hotkeys
**Rationale:** Standalone commit mode only touches the `main.rs` startup path — it is isolated from all the rendering and document work in earlier phases. Custom hotkeys must be last because they cross-cut every input handler and the config schema depends on all features being finalized. Custom hotkeys are conditional on complexity budget.
**Delivers:** `gitmedit` with no args opens TUI in a git repo, writes message, invokes `git commit -F <tmpfile>`, exits with git's exit code; optionally `~/.config/gitmedit/config.toml` overrides key bindings
**Uses:** `std::process::Command` (stdlib); `tempfile` (promoted dep); `toml 0.8` + `serde` + `dirs 6.0` + `crokey 1.4` (config path only)
**Implements:** `InvocationMode` enum in `main.rs`; `Cli.path: Option<PathBuf>`; git repo + staged changes detection; optional `config.rs`
**Avoids:** Terminal not restored before subprocess (Pitfall 1); exit code misattribution from git (Pitfall 2); key conflict with hardcoded save/cancel bindings (Pitfall 10); startup latency from multiple subprocesses (Pitfall 11)

### Phase Ordering Rationale

- IO-06 fix precedes everything: the alternate screen behavior change alters how all rendering is perceived and cannot be evaluated after the fact.
- Plain editor default precedes nano chrome: it modifies `Document` internals; doing both simultaneously risks entangled changes to data paths that interact.
- Nano chrome precedes merge toolbar: the toolbar renders in the footer that chrome creates — this is a hard dependency confirmed in both FEATURES.md and ARCHITECTURE.md.
- Rebase features and merge toolbar are grouped: they share the "complete the git workflow" theme, and grouping them minimizes context-switching across the rebase state machine.
- Standalone commit and custom hotkeys are last: standalone is an isolated new code path; hotkeys require all features to be stable so the config schema does not churn.

### Research Flags

Phases needing implementation-time care (not pre-phase research sprints — the pitfalls are documented, just attention required):
- **Phase 4 (Rebase reordering):** Before writing the swap logic, write a focused unit test for `pick A / # comment / pick B` + move-B-up to validate the `selectable_indices` dual-swap behavior empirically. Treat as a design spike before full implementation.
- **Phase 5 (Standalone commit):** Subprocess error handling for `git commit -F` requires manual end-to-end testing with: no staged changes, empty message, and a commit hook that rejects. Verify exit codes and terminal state in each case.
- **Phase 5 (Custom hotkeys):** Confirm `crokey 1.4` resolves via `cargo add crokey@1.4` before implementing; if the version is absent, pin to latest resolved and update the config schema.

Phases with standard, well-documented patterns (no additional research needed):
- **Phase 1:** The IO-06 fix and Drop `.unwrap()` fix are single-line changes to known locations in `terminal.rs`.
- **Phase 2:** Architecture doc specifies the exact code change — `EditorMode` enum, flag on `Document`, no changes to `classify_line()` contract.
- **Phase 3:** ratatui 3-slot layout is a standard pattern with official documentation; architecture doc provides the exact constraint array.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All existing deps locked. New deps verified via crates.io, GitHub, and official sources. Crokey 1.4 version number from single source — confirm with `cargo add` before committing. |
| Features | HIGH | Table stakes from official git docs and reference tool analysis (git-interactive-rebase-tool). MERGE_MSG comment format is MEDIUM due to git version variance. Custom hotkeys complexity is well-characterized. |
| Architecture | HIGH | All findings from direct code inspection of v1.0 `src/*.rs`. Exact struct fields, function names, and change locations are specified. No inference required. |
| Pitfalls | HIGH | Critical pitfalls identified from direct code reading (`.unwrap()` in Drop at terminal.rs:66, `selectable_indices` structure in app.rs). Windows ConPTY behavior is MEDIUM — confirmed from crossterm docs but not empirically tested on Windows. |

**Overall confidence:** HIGH

### Gaps to Address

- **Crokey 1.4 version confirmation:** STACK.md notes the version number comes from a single source (GitHub Cargo.toml). Run `cargo add crokey@1.4` before building the hotkey config feature. If it fails to resolve, `cargo add crokey` to get latest and pin that version.
- **`selectable_indices` behavior with interleaved Comment lines:** Architecture doc specifies the dual-swap behavior but acknowledges the subtle path. Write a unit test for `pick A / # comment / pick B` before implementing the general reorder case. This is a validation step, not a research gap.
- **MERGE_MSG format across git versions:** Confirm the defensive parser handles actual MERGE_MSG files from git 2.39 and 2.47 before merging the merge toolbar. Specifically test: merge with conflicts, fast-forward merge (no conflict block), and an octopus merge.
- **Windows Shift+Up/Down key events:** Architecture doc flags these as reliable on macOS/Linux but needing empirical verification on Windows ConPTY. Deferred to post-ship cross-platform testing.

## Sources

### Primary (HIGH confidence)
- Direct code inspection: all 7 modules in `src/` (main.rs, app.rs, context.rs, document.rs, renderer.rs, terminal.rs, writer.rs) — architecture findings are HIGH confidence
- [git-interactive-rebase-tool GitHub](https://github.com/MitMaro/git-interactive-rebase-tool) — reference for rebase TUI features including exec editing and reordering
- [GNU nano official docs](https://www.nano-editor.org/dist/latest/nano.html) — 4-area layout confirmed (title bar, edit window, status bar, two help lines)
- [Git rebase documentation](https://git-scm.com/docs/git-rebase) — exec line format, todo file structure
- [Git merge documentation](https://git-scm.com/docs/git-merge) — MERGE_MSG format, conflict comment block
- [Git commit documentation](https://git-scm.com/docs/git-commit) — COMMIT_EDITMSG, editor invocation, -F flag
- [git fmt-merge-msg documentation](https://git-scm.com/docs/git-fmt-merge-msg) — merge message comment block structure
- [ratatui.rs — Testing with insta snapshots](https://ratatui.rs/recipes/testing/snapshots/) — official TestBackend + insta pattern
- [docs.rs — std::process::Command](https://doc.rust-lang.org/std/process/struct.Command.html) — stdlib subprocess, no version concerns
- [Crossterm: Windows key events press/release](https://github.com/crossterm-rs/crossterm) — duplication on Windows ConPTY confirmed
- [crates.io — toml](https://crates.io/crates/toml), [dirs](https://docs.rs/crate/dirs/latest), [crokey](https://lib.rs/crates/crokey), [insta](https://crates.io/crates/insta), [tempfile](https://docs.rs/crate/tempfile/latest)

### Secondary (MEDIUM confidence)
- [dystroy.org — Manage keybindings in Rust TUI](https://dystroy.org/blog/keybindings/) — crokey TOML hotkey config pattern; used in broot production
- [Ratatui keybinding discussion](https://github.com/ratatui/ratatui/discussions/627) — community patterns for configurable hotkeys in ratatui apps
- [JetBrains IDEA rebase reordering data loss](https://youtrack.jetbrains.com/issue/IDEA-203688/data-loss-git-rebase-interactive-will-loses-commits-when-reordering) — real-world precedent for selectable_indices-type bugs in mature tooling
- [git commit empty message abort behavior](https://github.com/desktop/desktop/issues/10462) — exit code 1 for empty message; widely reproduced in GitHub Desktop

### Tertiary (LOW confidence)
- WebSearch results on nano layout, standalone commit workflow, cross-platform terminal compatibility — consistent with primary sources but not formally verified

---
*Research completed: 2026-04-07*
*Ready for roadmap: yes*
