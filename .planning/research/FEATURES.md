# Feature Research

**Domain:** TUI git commit/rebase editor (replaces core.editor)
**Researched:** 2026-04-07
**Confidence:** HIGH for table stakes (well-understood git contracts); MEDIUM for differentiators (fewer prior art examples in this narrow niche)

---

## Context: v1.1 Scope

This document has been updated to cover the v1.1 milestone features layered on top of the shipped v1.0 base.
Features already validated in v1.0 are listed as completed in the dependency map and prioritization matrix.
The new features under investigation:

1. Plain editor default — remove comment protection/read-only logic
2. Nano-style chrome — header bar, filename, bottom command bar
3. Standalone commit mode — `gitmedit` with no args invokes `git commit -F <tmpfile>`
4. Merge commit toolbar — parse MERGE_MSG comment block for conflict/affected files
5. Fix IO-06 — alternate screen regression
6. Rebase line reordering — move commits up/down in todo
7. Exec line argument editing — edit the shell command in `exec` lines
8. Cross-platform testing — Windows terminal, iTerm2
9. Custom hotkey configuration — user-definable key bindings

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist in any standalone editor. Missing these makes the product feel incomplete or broken.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Open file from argv[1] and display editable buffer | Git's core.editor contract | LOW | Already shipped v1.0 |
| Save (Ctrl+S) / Cancel (Esc) with correct exit codes | Git contract requirement | LOW | Already shipped v1.0 |
| Preserve comment lines on save | Git uses `#` comments for context | LOW | Already shipped v1.0 |
| Handle COMMIT_EDITMSG, MERGE_MSG, git-rebase-todo | Core git invocation paths | MEDIUM | Already shipped v1.0 |
| Git context detection by filename | Required to choose correct UI mode | LOW | Already shipped v1.0 |
| Visible keybinding hints at bottom | Nano popularized this; users from nano expect it | LOW | Already shipped v1.0 |
| **All lines editable by default** | Any plain text editor allows editing all content | LOW | v1.0 had comment protection. v1.1 removes this — plain editor mode for non-git-rebase-todo contexts |
| **Visible filename and directory in header** | Nano shows this; users expect it to know what file they are editing | LOW | v1.1. Title bar: left=version, center=filename, right=modified flag — this is the nano layout convention |
| **Bottom shortcut bar with context-relevant commands** | Nano's two-line shortcut bar is the de facto standard for terminal editors | LOW | v1.1. Show ^S Save / Esc Cancel / ^H Help at minimum |
| **Rebase lines can be reordered** | git-interactive-rebase-tool does this; any rebase TUI is expected to support it | MEDIUM | v1.1. Move selected line up/down with Alt+Up/Alt+Down or equivalent |

### Differentiators (Competitive Advantage)

Features that set gitmedit apart. Not required by the git editor contract, but aligned with the "git-aware minimal editor" value proposition.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Git context detection (all modes) | Adjusts UI for commit/merge/rebase/squash contexts | MEDIUM | Already shipped v1.0 |
| Subject line character counter (50/72 rule) | Live feedback on commit hygiene; no plain editor shows this | LOW | Already shipped v1.0 |
| Rebase action cycling (pick/squash/fixup/drop) | Fast structured editing without text manipulation | MEDIUM | Already shipped v1.0 |
| Squash message dual-pane display | Context for accumulated squash messages | MEDIUM | Already shipped v1.0 |
| Word-wrapped rebase subjects | Readability improvement over truncation | LOW | Already shipped v1.0 |
| **Standalone commit mode** | Run `gitmedit` in a repo with no args to commit staged changes — no git wrapper needed | HIGH | v1.1. Invokes `git commit -F <tmpfile>` after user writes a message. Requires: checking for git repo, checking for staged changes, creating tmpfile, invoking git. The user experience should mirror `git commit` (editor opens, user writes, saves, git commits) |
| **Merge commit toolbar** | Display conflict file count and affected branch info in status bar during MERGE_MSG editing | MEDIUM | v1.1. Git's MERGE_MSG comment block format: lines starting with `# Conflicts:` followed by `#\t<filepath>`. Parse these, show "3 conflicts: file1, file2..." in toolbar. Significant UX help; no other plain editor does this |
| **Exec line inline editing** | Edit the shell command on `exec` lines in rebase todo without breaking line structure | MEDIUM | v1.1. The exec line format is `exec <shell-command>`. Users need to change the command. Dedicated editing mode or inline editing with Enter to confirm. git-interactive-rebase-tool (the reference implementation in Rust) supports this |
| **Custom hotkey configuration** | Power users want to remap keys to muscle memory from other tools | HIGH | v1.1. TOML config file at `~/.config/gitmedit/config.toml`. Map action names to key combinations. Default file created on first run. Adds complexity but unlocks accessibility and power-user adoption |
| Comment block styling | `#` lines rendered dim/grey so they feel non-editable contextually | LOW | Already shipped v1.0 |
| Help overlay (Ctrl+H) | Full keybinding reference without leaving the editor | LOW | Already shipped v1.0 |

### Anti-Features (Commonly Requested, Often Problematic)

Features to deliberately exclude.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Vim/Emacs keybindings | Power users want muscle memory | Contradicts the "not vim" positioning; adds config surface | Document that vim users should keep using vim |
| Plugin system | Developers always want hooks | Violates single-purpose philosophy; startup overhead | Keep behavior fixed |
| Syntax highlighting of commit bodies | Seems useful | Commit messages are prose, not code; adds complexity for zero utility | Apply styling only to structural git elements |
| Auto-stage on standalone commit | Convenience workflow | Dangerous — silent auto-staging can commit unintended changes | Standalone mode should require staged changes to exist before opening |
| Configuring colors/themes | Personalization | Config file complexity disproportionate to value for a commit editor | Fixed defaults; use terminal theme for color preferences |
| Spell checking | Help writing good messages | Dictionary dependency; increases binary size and startup | Out of scope |
| Git log viewing in commit mode | Convenient reference | Scope creep; opens git subprocess, adds UI complexity | Users open a second terminal pane |
| Conflict resolution UI | Logical extension of merge context | Full conflict resolution is a separate product surface (lazygit, neovim fugitive); not a commit editor | Show conflict file list for awareness only; resolution stays outside gitmedit |

---

## Feature Detail: Standalone Commit Mode

**How it works:**
1. User runs `gitmedit` in a git repo with no file argument.
2. gitmedit checks: is this a git repo? (`git rev-parse --git-dir` or check for `.git/`). If not, exit with error.
3. gitmedit checks: are there staged changes? (`git diff --cached --quiet` — exit code 1 = staged changes exist). If nothing staged, warn and abort.
4. gitmedit creates a temporary file (or uses a well-known path like `/tmp/gitmedit-XXXXXXXX.txt` or reuses `.git/COMMIT_EDITMSG`).
5. gitmedit optionally pre-populates the file with the commit template if one is configured (skip in v1.1).
6. User writes commit message, saves (Ctrl+S).
7. gitmedit invokes `git commit -F <tmpfile>` via `std::process::Command`.
8. gitmedit displays the output of git commit (success/failure).
9. On cancel (Esc), gitmedit exits without invoking git commit.

**Edge cases:**
- No staged changes: show a clear error, do not open editor.
- Not in a git repo: show clear error.
- Git commit fails (hooks reject, etc.): surface the error to the user; do not silently discard it.
- Concurrent commits (another process): git's own locking handles this.

**Confidence:** MEDIUM. The workflow is straightforward but git subprocess error handling needs careful testing.

---

## Feature Detail: Nano-Style Chrome

**Nano's actual layout (confirmed from official docs, HIGH confidence):**

```
[ GNU nano 7.2 ]                     filename.txt                     [ Modified ]
─────────────────────────────────── edit area ───────────────────────────────────
                                (cursor, content)
─────────────────────────────────────────────────────────────────────────────────
                              [ status bar: messages, prompts ]
^G Help     ^O Write Out    ^F Where Is    ^\ Replace    ^T Execute
^X Exit     ^R Read File    ^W Where Was   ^U Paste       ^J Justify
```

- Title bar (line 1): three sections — left (app name/version), center (filename), right (modified state)
- Edit area (full height minus chrome): scrollable text
- Status bar (3rd from bottom): one-line area for messages, prompts, search input
- Help lines (bottom 2 lines): visible shortcut labels — caret (^) prefix means Ctrl

**gitmedit v1.1 adaptation:**

- Header bar: `gitmedit` or `gitmedit v1.1` on left, git context mode in center (e.g. `commit`, `merge`, `rebase`), filename (basename) on right.
- No status bar line needed if errors use inline rendering — keep it simple.
- Bottom command bar: two lines, context-sensitive. In commit mode: `^S Save   Esc Cancel   ^H Help`. In rebase mode: add `p Pick   s Squash   f Fixup   d Drop   Alt+↑ Move Up   Alt+↓ Move Down`.
- The command bar should NOT scroll or animate — static text, rendered each frame.

**Complexity:** LOW. Pure rendering addition. Uses existing ratatui layout system.

---

## Feature Detail: Merge Commit Toolbar

**Git MERGE_MSG comment block format (MEDIUM confidence — confirmed from git docs and community sources):**

When a merge has conflicts, git writes a MERGE_MSG with a comment block like:
```
Merge branch 'feature-x'

# Conflicts:
#	src/main.rs
#	tests/integration.rs
```

When the merge is clean (fast-forward or no conflicts), the comment block may show the shortlog instead (depending on `merge.log` config). The `# Conflicts:` section only appears when there were actual conflicts.

**What to display:**
- Parse lines starting with `# Conflicts:` and the `#\t<path>` lines following it.
- Count conflict files, extract paths.
- Show in the header or status bar: e.g. `2 conflicts | src/main.rs, tests/integration.rs`.
- If no `# Conflicts:` section, show nothing extra (clean merge).
- Optionally show the branch being merged from the first line of MERGE_MSG (format: `Merge branch 'X'` or `Merge branch 'X' into Y`).

**Complexity:** MEDIUM. Requires extending the MERGE_MSG parser that already exists in v1.0.

---

## Feature Detail: Rebase Line Reordering

**Expected behavior (confirmed from git docs and reference tools, HIGH confidence):**

In interactive rebase, the order of lines in git-rebase-todo determines the order commits will be applied. Moving a line up means that commit runs earlier; moving a line down means later.

Standard UX conventions from git-interactive-rebase-tool (the reference Rust implementation):
- Select a line (cursor on it).
- Press a keybinding to move it up (one position) or down (one position).
- Visual feedback: the line moves instantly, cursor follows.
- No drag-and-drop (terminal editor; keyboard-only).
- Reordering may cause conflicts on git's side, but that is git's problem — the editor just writes the new order.

**Implementation notes:**
- The underlying data structure is a `Vec<RebaseLine>`. Swap adjacent elements.
- Keybinding choice: `Alt+Up` / `Alt+Down` or `Shift+Up` / `Shift+Down` or `K` / `J` (vim-style, but only in rebase mode).
- gitmedit should use something distinct from navigation to prevent accidental reorders.
- Recommended: `Alt+Up` / `Alt+Down` — natural, discoverable, distinct from cursor navigation.

**Dependency:** Requires the existing rebase table to track cursor position per-row (already exists from v1.0 action cycling).

**Complexity:** MEDIUM. Vec swap is trivial; the care is in keybinding choice and ensuring the render loop shows the new order immediately.

---

## Feature Detail: Exec Line Argument Editing

**Exec line format (HIGH confidence, from git docs):**

```
exec <shell-command>
```

Example: `exec make test` or `exec npm run build`.

The hash field is absent — exec lines have: `exec` keyword + space + arbitrary shell command.

**Expected behavior:**
- User selects an exec line in the rebase table.
- Presses Enter (or a dedicated keybinding like `e` for edit) to open an inline editor for the command text.
- The command text becomes an editable single-line input field.
- User edits the command; Enter to confirm, Esc to cancel.
- On confirm, the exec line is updated in the Vec; the table re-renders.

**Reference:** git-interactive-rebase-tool supports this — it opens a prompt/input line at the bottom for exec command editing.

**Complexity:** MEDIUM. Requires a single-line text input widget (either a narrow ratatui-textarea instance or a custom input field). The tricky part is integration with the rebase table state machine — switching between table navigation mode and inline edit mode.

---

## Feature Detail: Custom Hotkey Configuration

**Common pattern in Rust TUI apps (MEDIUM confidence):**

Config file location: `~/.config/gitmedit/config.toml` (XDG-compliant).

Typical TOML keybinding section:
```toml
[keys]
save = "ctrl+s"
cancel = "esc"
help = "ctrl+h"
rebase_pick = "p"
rebase_squash = "s"
rebase_fixup = "f"
rebase_drop = "d"
rebase_move_up = "alt+up"
rebase_move_down = "alt+down"
```

**Design constraints:**
- Default config generated on first run if file is absent.
- Parse key string to crossterm KeyEvent at startup.
- Invalid key strings should warn (stderr or log) and fall back to default.
- No hot-reload needed — read once at startup.
- Do NOT require a config file to work — all defaults are hardcoded, config only overrides.

**Complexity:** HIGH. Not because the file format is complex, but because it requires threading the key mapping through every input handler. Every `match key_event { KeyCode::Ctrl('s') => ... }` pattern becomes a lookup in the key map. This is an architectural change, not a feature add.

**Alternative considered:** Hardcoded keys only (no config). For a minimal commit editor used in brief sessions, custom hotkeys may be over-engineered. The v1.1 scope includes it, but it should be implemented last (after all other features are working) and only if time/complexity allows.

---

## Feature Detail: Alternate Screen Fix (IO-06)

**The regression:** v1.0 inadvertently uses `EnterAlternateScreen` (crossterm), which causes the TUI to render in an alternate terminal buffer. This erases the TUI output when the editor exits, unlike nano's behavior where the edited content scrolls off in the main buffer.

**Expected behavior (nano model):**
- Editor renders in the main terminal scroll buffer.
- On exit, cursor returns to the normal position; previous content stays visible above.
- No flickering swap between buffers.

**Implementation:** Use `crossterm::terminal::disable_raw_mode` + direct stdout rendering without `EnterAlternateScreen` / `LeaveAlternateScreen`. Ratatui supports this via `CrosstermBackend` with a plain stdout handle.

**Complexity:** LOW. A known regression with a well-understood fix. Main risk: subtle re-rendering artifacts in terminals that behave differently without alternate screen.

---

## Feature Dependencies (v1.1)

```
[v1.0 base: file read/write, TUI, text editing, git context, rebase table]

[Nano-style chrome]
    └──requires──> [v1.0 TUI rendering]
    └──extends──> [All modes: commit, merge, rebase, squash]

[Plain editor default]
    └──requires──> [v1.0 text editing]
    └──modifies──> [Comment protection logic — remove it for non-rebase contexts]

[Standalone commit mode]
    └──requires──> [v1.0 text editing + file write]
    └──requires──> [git repo detection]
    └──requires──> [staged changes detection]
    └──requires──> [subprocess invocation (git commit -F)]
    └──optionally-benefits-from──> [Nano-style chrome] (shows context = "standalone commit")

[Merge commit toolbar]
    └──requires──> [v1.0 MERGE_MSG parser]
    └──requires──> [Nano-style chrome] (toolbar lives in header or status bar)
    └──extends──> [v1.0 git context detection]

[Rebase line reordering]
    └──requires──> [v1.0 rebase table with cursor]
    └──requires──> [v1.0 rebase action cycling keybindings as pattern reference]

[Exec line argument editing]
    └──requires──> [v1.0 rebase table]
    └──requires──> [single-line input widget]

[Custom hotkey configuration]
    └──requires──> [ALL other features to be stable first]
    └──affects──> [Every input handler in the application]
    └──requires──> [Config file I/O + TOML parsing]

[IO-06 alternate screen fix]
    └──requires──> [v1.0 terminal initialization code]
    └──independent──> [all other v1.1 features — fix this first]
```

### Dependency Notes

- **IO-06 fix is a prerequisite for meaningful testing of chrome changes.** The alternate screen regression makes it impossible to correctly evaluate how nano-style chrome looks and feels. Fix this before building chrome.
- **Nano-style chrome is a prerequisite for merge toolbar.** The toolbar content has nowhere to live without the chrome layout.
- **Standalone commit mode is independent.** It does not depend on chrome changes. Can be built in parallel but will benefit from chrome for context display.
- **Custom hotkey configuration should be last.** It requires all other features to be finalized (otherwise key bindings keep changing during implementation, causing churn in the config schema).
- **Rebase reordering and exec editing are independent of each other** but both depend on the existing rebase table.

---

## MVP Definition for v1.1

### Must Ship

| Feature | Rationale |
|---------|-----------|
| IO-06 alternate screen fix | Regression from v1.0; blocks correct evaluation of all chrome changes |
| Plain editor default | Simplification; removes confusing read-only behavior on comment lines |
| Nano-style chrome | Core UX goal of the milestone; foundation for merge toolbar |
| Standalone commit mode | New user-facing capability; the headline feature of this milestone |
| Merge commit toolbar | High signal-to-noise ratio; small implementation relative to value |
| Rebase line reordering | Table stakes for any interactive rebase TUI |
| Exec line argument editing | Completes the rebase feature set |

### Should Ship if Complexity Allows

| Feature | Rationale |
|---------|-----------|
| Custom hotkey configuration | Valuable for power users; but high implementation cost and should come last |

### Defer

| Feature | Reason |
|---------|--------|
| Cross-platform testing (Windows) | Testing effort, not implementation; can be done as a followup after other features ship |
| Undo/redo (EDIT-07) | Known gap; adds state management complexity; low priority for short-session commit editor |

---

## Feature Prioritization Matrix (v1.1)

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| IO-06 alternate screen fix | HIGH (unblocks chrome) | LOW | P1 — first |
| Plain editor default | MEDIUM | LOW | P1 |
| Nano-style chrome (header + command bar) | HIGH | LOW | P1 |
| Standalone commit mode | HIGH | MEDIUM | P1 |
| Merge commit toolbar | MEDIUM | MEDIUM | P2 |
| Rebase line reordering | HIGH | MEDIUM | P2 |
| Exec line argument editing | MEDIUM | MEDIUM | P2 |
| Custom hotkey configuration | MEDIUM | HIGH | P3 — last |
| Cross-platform testing | LOW (now) | HIGH (effort) | P3 — deferred |

---

## Competitor Feature Analysis (v1.1 features)

| Feature | nano | git-interactive-rebase-tool | lazygit | gitmedit v1.1 target |
|---------|------|----------------------------|---------|----------------------|
| Nano-style chrome (header + command bar) | Yes — reference implementation | No | No | Yes — modeled on nano |
| Standalone commit mode | No (nano is just an editor) | No | Yes (full workflow) | Yes — lightweight version |
| Merge conflict file toolbar | No | N/A | Yes (full UI) | Yes — parsed from MERGE_MSG |
| Rebase line reordering | No | Yes | Yes | Yes |
| Exec line editing | No | Yes (inline prompt) | Yes | Yes — inline prompt |
| Custom hotkeys | Yes (nanorc) | Yes (config file) | Yes (config.yml) | Yes — TOML config |
| Alternate screen behavior | No alternate screen | Yes alternate screen | Yes alternate screen | No alternate screen (nano model) |

---

## Sources

- [git-interactive-rebase-tool GitHub](https://github.com/MitMaro/git-interactive-rebase-tool) — reference for rebase TUI features including exec editing and reordering (HIGH confidence)
- [GNU nano official docs layout](https://www.nano-editor.org/dist/latest/nano.html) — confirmed 4-area layout: title bar, edit window, status bar, two help lines (HIGH confidence)
- [Git rebase documentation](https://git-scm.com/docs/git-rebase) — exec line format, todo file structure (HIGH confidence)
- [Git merge documentation](https://git-scm.com/docs/git-merge) — MERGE_MSG format, conflict comment block (HIGH confidence)
- [Git commit documentation](https://git-scm.com/docs/git-commit) — COMMIT_EDITMSG, editor invocation, -F flag (HIGH confidence)
- [Ratatui keybinding discussion](https://github.com/ratatui/ratatui/discussions/627) — community patterns for configurable hotkeys in ratatui apps (MEDIUM confidence)
- [Manage keybindings in Rust TUI — dystroy](https://dystroy.org/blog/keybindings/) — TOML hotkey config patterns (MEDIUM confidence)
- [Git fmt-merge-msg documentation](https://git-scm.com/docs/git-fmt-merge-msg) — merge message comment block structure (HIGH confidence)
- WebSearch: nano layout, standalone commit workflow, cross-platform terminal compatibility — consistent with above sources (MEDIUM confidence)

---

*Feature research for: TUI git commit/rebase editor — v1.1 milestone*
*Researched: 2026-04-07*
