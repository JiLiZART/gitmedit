# Domain Pitfalls

**Domain:** TUI git editor (Rust, ratatui + crossterm) — v1.1 Feature Addition
**Researched:** 2026-04-07
**Confidence:** HIGH for git contract issues; HIGH for codebase-specific integration risks (based on direct code inspection); MEDIUM for cross-platform terminal behavior

---

## Context: Why This Document Exists

v1.0 pitfalls covered building the system from scratch. v1.1 pitfalls are different in character: they are about adding features to a working system without breaking what already works. The risks cluster around three categories:

1. **Integration regressions** — new features break existing modes
2. **Process spawning** — standalone commit mode introduces a subprocess that must not leak or fail silently
3. **Mutation safety** — rebase reordering mutates the in-memory todo list; ordering bugs corrupt git history

---

## Critical Pitfalls

### Pitfall 1: Standalone Commit Mode Leaves Terminal in Raw Mode When `git commit -F` Fails

**What goes wrong:**
`gitmedit` (no args) spawns `git commit -F <tmpfile>` as a child process. If the child process fails (non-zero exit), the code must restore the terminal before printing an error message to stderr. If the TUI was still active when the subprocess ran — or if error handling skips the terminal guard drop — the user sees garbled output in raw mode with no way to recover except running `reset`.

**Why it happens:**
In the existing code, terminal restore is done by `drop(guard)` explicitly before `process::exit(0)`. If a new `git commit -F` call path returns an error _after_ the TUI exits but _before_ the guard is dropped, or if the subprocess return is handled with `?` propagation that bypasses explicit guard cleanup, the terminal guard may not drop correctly. `Drop` on a `TerminalGuard` does call `LeaveAlternateScreen` and `disable_raw_mode`, but `Drop::drop` calls `.unwrap()` on `crossterm::execute!` — a panic inside Drop causes undefined behavior.

Specifically, in `terminal.rs` line 66:
```rust
crossterm::execute!(
    self.terminal().backend_mut(),
    LeaveAlternateScreen,
    DisableMouseCapture
)
    .unwrap(); // <-- panics inside Drop if crossterm fails
```
A panic inside Drop is UB territory in Rust. Replace with `let _ = ...`.

**Consequences:**
- User terminal stuck in raw mode after subprocess failure
- Error message rendered unreadable or invisible
- User must kill terminal session

**Prevention:**
- Drop the terminal guard explicitly via `drop(guard)` before calling any subprocess
- Never propagate errors with `?` from code that still holds the terminal guard
- Fix the `.unwrap()` in `TerminalGuard::Drop` to `let _ = crossterm::execute!(...)` (pre-existing debt)
- Flush stderr after writing error messages when terminal has been restored

**Detection:**
Trigger `git commit -F` in a repo with no staged changes; verify the terminal is fully restored and the error message is readable.

**Phase:** Standalone commit mode (whichever phase implements `gitmedit` no-args mode)

---

### Pitfall 2: Standalone Commit Mode — Empty Message Silently Aborts with Exit Code 1 Misattributed

**What goes wrong:**
When `gitmedit` spawns `git commit -F <tmpfile>` and the user saves an empty message (or a message that is only comment lines), git exits with code 1 and prints "Aborting commit due to empty commit message" to stderr. The `gitmedit` process receives exit code 1 from the child and must distinguish this user-initiated abort from a real git error (permissions, hooks, no staged changes).

If the code treats any non-zero exit from `git commit` as an internal error and prints a generic "git commit failed" message, the user sees confusing output. If it treats it as success and exits 0 itself, callers get wrong signals.

**Why it happens:**
`std::process::Command::status()` returns the exit code but not stderr. Git uses exit code 1 for both "empty message abort" and "hook rejected commit" and "nothing to commit." The distinction lives in stderr text, which requires capturing separately.

**Consequences:**
- Confusing UX: user types a message, saves, sees "git commit failed" with no useful detail
- Or worse: empty message causes git to abort but gitmedit exits 0, no error shown

**Prevention:**
- Capture both stdout and stderr from the subprocess: use `.output()` not `.status()`
- Pass git's stderr through to the user's terminal after restoring it
- Exit with the same code git returned: if git exits 1, gitmedit exits 1; do not swallow the code
- Document that "nothing to commit" from git is a normal non-error user path, not an application bug

**Detection:**
Run `gitmedit` with no staged changes. Run it with empty message. Run it with a commit hook that rejects. Verify exit codes and stderr messages in each case.

**Phase:** Standalone commit mode

---

### Pitfall 3: "Plain Editor" Refactor Breaks Squash Mode — `editable_lines()` Assumption

**What goes wrong:**
The "plain editor default" feature removes the `ContentLine::Comment` / `ContentLine::ConflictMarker` distinction from what is shown in the `TextArea` — all lines become editable. This is the right behavior for the generic editor. However, the squash mode in `App::new` explicitly depends on `document.editable_lines()` returning only non-comment lines to populate the `TextArea`. If the refactor makes `editable_lines()` return all lines (including squash header comments), the squash log separator logic breaks: the squash header will appear editable when it should remain read-only.

Similarly, `Document::serialize()` currently interleaves `textarea_lines` content back at `content_idx` positions. If the plain-editor mode puts all lines into the textarea, the `content_idx` counter will not match the `lines` vector structure, producing corrupted output.

**Why it happens:**
`Document` has two concerns: (a) storing a typed representation of all lines and (b) knowing which lines are editable. The plain editor feature wants "all lines are editable," but squash mode wants "only non-comment lines are editable." These requirements conflict within the same struct.

**Consequences:**
- Squash mode edits appear to work but write garbled content back
- Squash header comment lines become editable (regression in SQUASH-01 through SQUASH-04)
- Merge conflict markers become editable (regression in MERGE-01 through MERGE-03)

**Prevention:**
- Implement plain-editor mode as a mode flag on `Document` or as a separate code path in `App::new` — do NOT change `Document::parse` to classify comment lines as `Content`
- After the refactor, run all existing squash-mode tests: `test_app_squash_mode_has_log`, `test_app_squash_mode_editable`, `test_app_squash_serialized_content`
- Keep the round-trip tests: `test_roundtrip_preserves_comments`, `test_roundtrip_preserves_conflict_markers`

**Detection:**
Run `cargo test` after any change to `Document::parse` or `editable_lines()`. Try `git merge` with conflict markers and save — verify markers are preserved verbatim.

**Phase:** Plain editor default (refactor phase)

---

### Pitfall 4: Nano Chrome Layout Breaks Rebase Table Rendering Due to Constraint Miscalculation

**What goes wrong:**
The current renderer in `renderer.rs` splits the terminal into `[Constraint::Min(0), Constraint::Length(1)]` — content area and one-line status bar. Adding nano-style chrome (header row at top + command bar at bottom) changes this to at minimum `[Length(1), Min(0), Length(1)]`. The rebase table renderer (`render_rebase_table`) computes visible height from `chunks[0]`, and the content renderer (`render_content`) computes `visible_height` from `area.height`. If the chrome layout changes which chunk index maps to which area, the scroll offset calculations will be off by the chrome height.

In the current code (`renderer.rs` lines 59-63):
```rust
let visible_height = area.height as usize;
let scroll_top = if cursor_full_row != usize::MAX && cursor_full_row >= visible_height {
    cursor_full_row - visible_height + 1
} else { 0 };
```
This uses the passed `area`, so it is correct — but only if every render call path passes the right reduced `area` after chrome is subtracted. A missed render path that still passes the full frame will scroll incorrectly.

**Why it happens:**
There are five render dispatch paths: `render_rebase_table`, `render_squash_mode`, `render_content`, `render_rebase_status_bar`, `render_squash_status_bar`, `render_status_bar`. Adding chrome requires updating all of them to receive the shrunken content area. Forgetting one path causes that mode to render incorrectly.

**Consequences:**
- Status bar or chrome overlap the content area
- Cursor scroll behaves incorrectly in one or more modes
- Off-by-one in visible_height causes premature or late scrolling

**Prevention:**
- The Layout split for chrome must be defined once in the top-level `Renderer::render` function and passed to all sub-renderers — do not redefine it per sub-renderer
- Write a test that checks that all layout chunks sum to `frame.area()` after splitting
- Add the header in a `Constraint::Length(1)` or `Length(2)` chunk at top; add command bar as `Length(1)` at bottom; content gets `Min(0)` in between
- After implementing, verify in rebase mode (table), squash mode, commit mode, and merge mode that all visible heights are correct

**Detection:**
Open a rebase todo with 20+ commits. Verify cursor scrolling works correctly. Open a commit message. Verify the command bar does not overlap text.

**Phase:** Nano chrome implementation

---

### Pitfall 5: Rebase Reordering Corrupts `selectable_indices` After Swap

**What goes wrong:**
`App.selectable_indices` is a `Vec<usize>` mapping logical selection index (0-based sequential) to physical index into `rebase_lines`. When two `RebaseLine::Action` entries are swapped in `rebase_lines`, the physical indices in `selectable_indices` become stale. After a swap, `selectable_indices[0]` still points to the old index 0 in `rebase_lines`, not to wherever that action ended up after the swap.

Current structure in `app.rs`:
```
rebase_lines:        [Action(pick, abc), Action(pick, def), Comment(...), Action(drop, ghi)]
selectable_indices:  [0, 1, 3]  // indices into rebase_lines for Action lines
selected_rebase_idx: 0  // index into selectable_indices
```

If you swap `rebase_lines[0]` and `rebase_lines[1]`, `selectable_indices` must become `[0, 1, 3]` still (both moved Action lines are still at 0 and 1). This is fine IF reordering only swaps adjacent Action lines. But if the reorder operation moves an Action line past a Comment line (e.g., swap positions 1 and 2, where position 2 is a Comment), then after the swap: `rebase_lines[1]` is now the Comment, `rebase_lines[2]` is the Action. `selectable_indices` must be recomputed from scratch.

**Why it happens:**
The current MoveRebaseDown/MoveRebaseUp only moves the cursor (changes `selected_rebase_idx`), it does not reorder lines. When reordering is added, the implementation may naively swap two entries in `rebase_lines` without regenerating `selectable_indices`, which only breaks when non-Action lines (Comments) sit between the two Action lines being swapped.

**Consequences:**
- `selectable_indices.get(selected_rebase_idx)` returns a wrong index
- Tab cycles the wrong action
- Serialized content emits commits in wrong order
- Data loss: the user thinks they reordered but the output is different from what they saw

**Prevention:**
- After any reorder operation, regenerate `selectable_indices` from scratch by re-scanning `rebase_lines`
- This is a cheap O(n) scan — do not try to update `selectable_indices` incrementally
- Keep `selected_rebase_idx` pointing to a logical position (e.g., "follow the item that was moved"), update it after regeneration
- Write tests for swap where a Comment line sits between the two Action lines

**Detection:**
In a rebase with the structure: `pick A / # comment / pick B`, move B up past A. Verify the serialized output has B before A, not A before B. Verify the comment line stays in its relative position (or document the decision about where comments go during reorder).

**Phase:** Rebase line reordering

---

### Pitfall 6: Exec Line Editing Requires Special Serialization Path Already Present but Not Exposed

**What goes wrong:**
`exec` lines in `rebase_lines` use an empty `hash` field and store the entire command in `subject`. This is already handled correctly in `serialize_rebase_todo` (document.rs lines 132-135). However, if exec line editing is implemented by letting the user edit the `subject` field via a text input, and the edited text is stored back into `RebaseLine::Action { action: Exec, hash: "", subject: new_text }`, the serializer will produce `exec <new_text>` correctly.

The risk is that an editing UI might accidentally write the command into `hash` instead of `subject` (easy mistake since normal action lines use `hash` for the commit SHA), which would serialize as `exec <hash> <subject>` — a malformed exec line that git rejects.

**Why it happens:**
The `RebaseLine::Action` struct shares fields between exec and non-exec lines. The contract that `hash` is empty for exec lines is implicit, not enforced by the type system.

**Consequences:**
- `git rebase -i` receives a malformed exec line and aborts with a parse error
- Or the command `<hash>` is run as a shell command instead of `<subject>`

**Prevention:**
- When implementing exec editing UI, write back into `subject` only — never touch `hash` for exec lines
- Add a test: edit an exec line's command, serialize, verify no hash appears in the output
- Consider making the exec struct distinction explicit (separate variant or newtype), though this is optional given test coverage

**Phase:** Exec line argument editing

---

### Pitfall 7: Merge Commit Toolbar Parses Git-Generated Comments That Vary by Git Version and Locale

**What goes wrong:**
The merge commit toolbar is supposed to parse the MERGE_MSG comment block for conflict information (conflicted files, branch names). Git generates these comment lines using `git fmt-merge-msg` internally. The format is:

```
# Conflicts:
#	path/to/file.rs
```

The exact format (tab vs spaces, "Conflicts:" capitalization, presence of the section header at all) varies:
- Git < 2.35: may not include the `# Conflicts:` section in all merge scenarios
- Git with `--no-commit`: comments may be absent entirely
- Localized git builds: "Conflicts:" may be translated (rare but possible in enterprise environments)
- Fast-forward merges: MERGE_MSG has no conflict section
- Octopus merges: multiple conflict sections with different headers

If the parser hardcodes English strings or a specific indentation, it silently produces no results on git versions or configurations that differ.

**Why it happens:**
MERGE_MSG is a git-internal file with no formal schema specification. Git's behavior is determined by its C source code, not a documented format. The comment section content is produced by `fmt_merge_msg.c`.

**Consequences:**
- Toolbar shows no conflict information when git has a different format
- Toolbar shows wrong information (false positives) if format is misread
- No crash — just silent incorrect UI

**Prevention:**
- Parse the conflict section defensively: if the section is absent or unrecognized, show nothing (do not error)
- Test against at least git 2.39, 2.40, 2.47 MERGE_MSG outputs with actual merge commits
- Do not localize the parse — but do not crash on non-English strings
- Treat missing section as "no conflict info available" and hide the toolbar section rather than showing empty/wrong data

**Detection:**
Run a three-way merge with conflicts on git 2.39 and git 2.47. Compare the MERGE_MSG comment block. Verify the toolbar renders correctly in both cases. Test fast-forward merge (no comment block) — verify toolbar shows nothing rather than crashing.

**Phase:** Merge commit toolbar

---

## Moderate Pitfalls

### Pitfall 8: Windows ConPTY Key Events Fire Twice (Press + Release)

**What goes wrong:**
On Windows with crossterm, `Event::Key` fires for both `KeyEventKind::Press` and `KeyEventKind::Release`. The existing event loop in `main.rs` already filters `KeyEventKind::Press` events only (line 60: `kind: KeyEventKind::Press`), which is correct. However, any new event handling code added during v1.1 that matches on `Event::Key` without this filter will process every key twice on Windows.

**Why it happens:**
Windows ConPTY reports both press and release events; macOS/Linux only report press. The pattern guard `kind: KeyEventKind::Press` is in the outer match arm, so it covers all existing dispatch. New code added outside this arm (for example, for hotkey configuration that intercepts events differently) may miss the filter.

**Consequences:**
- Keystrokes execute actions twice on Windows
- Save triggers twice (harmless but confusing), cycle actions skip a step, etc.

**Prevention:**
- Never add key event handling outside the existing `Event::Key(KeyEvent { kind: KeyEventKind::Press, .. })` match arm
- For custom hotkey configuration: apply the same filter when reading user key input for hotkey recording

**Phase:** Cross-platform testing; Custom hotkey configuration

---

### Pitfall 9: Terminal Guard Drop Contains `.unwrap()` — Pre-existing Bomb

**What goes wrong:**
In `terminal.rs` line 66, `TerminalGuard::Drop` calls `.unwrap()` on a `crossterm::execute!` call. If the terminal backend is in an unexpected state when drop runs (e.g., stdout was closed, the backend was already released), this panics inside `Drop`. Panicking inside `Drop` during stack unwinding causes immediate process abort (`abort()` not unwind), which:
- Skips all remaining `Drop` implementations in the call stack
- May leave other resources uncleaned
- Produces a confusing "double panic" message

This is pre-existing debt that becomes a bomb when standalone commit mode is added, because standalone commit mode creates a new terminal lifecycle (open for editing, close before subprocess, reopen if needed for error display).

**Why it happens:**
The `.unwrap()` is a convenience shortcut. It was acceptable when `TerminalGuard` was only ever dropped once at process exit (where stdout is always valid). It becomes risky if multiple `TerminalGuard` instances are created in sequence or if the guard is dropped earlier than normal (as standalone commit mode requires).

**Prevention:**
- Replace `.unwrap()` with `let _ = crossterm::execute!(...)` in `TerminalGuard::Drop`
- This is a one-line fix that eliminates the panic-in-drop risk
- Do this before implementing standalone commit mode

**Phase:** This should be fixed in the first v1.1 phase, before standalone commit mode

---

### Pitfall 10: Custom Hotkey Configuration — Key Conflict Between `Ctrl+S` / `Ctrl+X` and System or Editor Defaults

**What goes wrong:**
Custom hotkey configuration allows users to remap keys. If the configuration format allows remapping `Ctrl+C`, `Ctrl+X`, `Ctrl+V`, or `Ctrl+Z`, users may create conflicts with the existing clipboard/undo bindings already handled in the main event loop (main.rs lines 169-195). Because the event loop handles these keys explicitly before falling through to `textarea.input(event)`, a custom hotkey config that tries to remap them would conflict with the hardcoded handlers.

Additionally, if `Ctrl+S` (save) is remappable, a user remapping it to something else will have no save key — the app becomes unexitable without Esc (cancel).

**Why it happens:**
The current keybindings are a mix of hardcoded handlers (save, cancel, help, undo, clipboard) and textarea-delegated handlers (character insertion, navigation). Custom hotkey config must interact with the hardcoded layer, not just the textarea layer.

**Prevention:**
- Separate "critical system keys" (save, cancel) from "configurable operation keys" (undo, clipboard, etc.)
- Disallow remapping save and cancel, or require explicit confirmation to remap them
- Validate the hotkey config at load time: if any remapped key conflicts with a hardcoded critical key, reject the config with a clear error
- Store custom hotkeys as a `HashMap<KeyBinding, Action>` and consult it in the event loop before the hardcoded checks

**Phase:** Custom hotkey configuration

---

### Pitfall 11: `read_comment_char()` Subprocess Called Before Terminal Init — Startup Latency Doubles If Git Is Slow

**What goes wrong:**
`read_comment_char()` in `document.rs` spawns a `git config --get core.commentchar` subprocess. This is currently called inside `App::new`, which runs before the terminal guard is acquired. If `git` is slow (network filesystem, slow PATH resolution, WSL), this adds latency before the first render. With standalone commit mode adding another subprocess (`git commit -F`), the number of git subprocesses per invocation grows, compounding startup latency.

**Why it happens:**
The current single-subprocess startup (17ms) is well within the <100ms target. Adding two or more subprocesses risks exceeding it on slow systems.

**Prevention:**
- Keep the `read_comment_char()` call where it is (before terminal init) — it must complete before parsing
- For standalone commit mode, spawn `git commit -F` after the TUI exits, not during startup
- Consider a 50ms timeout on the `read_comment_char()` subprocess: if it times out, fall back to `#` silently

**Phase:** Standalone commit mode

---

## Minor Pitfalls

### Pitfall 12: Nano Chrome Status Bar Truncation on Narrow Terminals

**What goes wrong:**
The nano chrome command bar (bottom line) shows key bindings like `^S Save  Esc Cancel  ^H Help`. On narrow terminals (< 40 columns), this text overflows and wraps or gets cut, potentially writing into the content area. Ratatui `Paragraph` will wrap if not constrained, or clip if using `Wrap::NoWrap` — but the clip happens inside the widget area, not outside it. The risk is different: if the chrome line is drawn at full terminal width but the text is too long for the terminal, it simply looks bad. It does not corrupt content.

**Prevention:**
- Use `Paragraph::new(text).wrap(Wrap { trim: true })` or truncate the string to `area.width` before rendering
- On very narrow terminals (< 30 cols), show shortened key names: `^S ^C ^H`

**Phase:** Nano chrome implementation

---

### Pitfall 13: Rebase Reorder — Moving a Line Past the Bottom Clamps on Last Action, Not Last Line

**What goes wrong:**
`MoveRebaseDown` currently clamps at `selectable_indices.len() - 1`. When reordering is added, "move down" should move the selected action line one position lower in the overall line list, past the next Action line. If the action line is already the last Action line, it cannot move down further. The clamp logic is already correct for navigation. The risk is that reorder-down may allow moving past the last Action line, resulting in an Action at the very end of the file — after all the comment lines that git appended. Git accepts this, but it looks confusing to the user.

**Prevention:**
- Decide and document whether Action lines can move past Comment lines or only past other Action lines
- If only past other Action lines: reorder-down should find the next Action line and swap positions with it
- If past any line: simpler swap logic, but document the behavior

**Phase:** Rebase line reordering

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Standalone commit mode | Terminal not restored before subprocess; `.unwrap()` in Drop | Fix Drop first; explicit `drop(guard)` before `git commit -F` call |
| Standalone commit mode | Subprocess exit code misattributed | Capture stderr with `.output()`; pass it through to user |
| Plain editor default | Breaks squash/merge modes via `Document` assumption changes | Keep `Document` logic unchanged; implement as mode in `App` |
| Nano chrome | Layout breaks scroll calculations in rebase/squash modes | Define layout once at top level; pass correct `area` to all render fns |
| Rebase reordering | `selectable_indices` becomes stale after swap past Comments | Regenerate `selectable_indices` after every reorder operation |
| Exec line editing | Subject/hash field confusion in `RebaseLine::Action` | Write to `subject` only; add serialization test |
| Merge toolbar | Git MERGE_MSG format varies by version | Parse defensively; fail silently when section is absent |
| Cross-platform | Windows key-press-release duplication | Keep new key handling inside existing `KeyEventKind::Press` arm |
| Custom hotkeys | Conflict with hardcoded save/cancel bindings | Separate critical keys from configurable keys at design time |

---

## Integration Regression Checklist

After each v1.1 feature is implemented, run this checklist before merging:

- [ ] `cargo test` passes (no regressions in document, app, or renderer tests)
- [ ] Commit mode: `git commit` opens editor; Ctrl+S saves; Esc aborts with exit code 1
- [ ] Merge mode: MERGE_MSG opens; conflict markers render styled; Ctrl+S saves verbatim
- [ ] Squash mode: squash header is read-only; editable portion is editable; serialized output includes both
- [ ] Rebase mode: tab cycles actions; up/down navigates; Ctrl+S writes valid todo file
- [ ] Standalone mode: `gitmedit` with no args in a git repo commits; empty message aborts correctly
- [ ] Reorder: moving a commit up/down produces correct serialized order
- [ ] Chrome: header and command bar visible in all modes; no overlap with content area
- [ ] Windows (if testing): no double-keypress actions

---

## Sources

- Codebase inspection: `/src/terminal.rs`, `/src/app.rs`, `/src/document.rs`, `/src/main.rs`, `/src/renderer.rs` — HIGH confidence (direct code reading)
- [Rust std::process::ExitStatus documentation](https://doc.rust-lang.org/std/process/struct.ExitStatus.html) — HIGH confidence (official docs)
- [std::process::Command output() error handling hazards](https://github.com/rust-lang/rust/issues/73126) — HIGH confidence (official tracking issue)
- [Crossterm: Windows key events fire press and release](https://github.com/crossterm-rs/crossterm) — HIGH confidence (official crossterm docs, confirmed in search results)
- [ratatui layout documentation](https://docs.rs/ratatui/latest/ratatui/layout/index.html) — HIGH confidence (official docs)
- [git-fmt-merge-msg documentation](https://git-scm.com/docs/git-fmt-merge-msg) — HIGH confidence (official git docs)
- [git-rebase documentation](https://git-scm.com/docs/git-rebase) — HIGH confidence (official git docs)
- [JetBrains IDEA data loss on rebase reordering](https://youtrack.jetbrains.com/issue/IDEA-203688/data-loss-git-rebase-interactive-will-loses-commits-when-reordering) — HIGH confidence (reproduced bug in mature tooling)
- [git commit empty message abort behavior](https://github.com/desktop/desktop/issues/10462) — HIGH confidence (widely reproduced issue in GitHub Desktop)
- [git-commit documentation: -F flag](https://git-scm.com/docs/git-commit) — HIGH confidence (official git docs)

---

*Pitfalls research for: gitmedit v1.1 — Adding standalone commit, editor simplification, nano chrome, merge parsing, rebase reordering*
*Researched: 2026-04-07*
