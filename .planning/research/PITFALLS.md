# Pitfalls Research

**Domain:** TUI git editor (Rust, `core.editor` replacement)
**Researched:** 2026-03-18
**Confidence:** HIGH for git contract issues; MEDIUM for TUI-specific issues

---

## Critical Pitfalls

### Pitfall 1: Terminal Not Restored After Panic

**What goes wrong:**
If the application panics while terminal raw mode is active (which is the entire time the TUI is running), the terminal is left in raw mode after the process exits. The user's shell becomes unusable — arrow keys print escape sequences, Enter does not work, output is garbled. The user must kill the terminal session entirely.

**Why it happens:**
`crossterm::terminal::disable_raw_mode()` is not called automatically on panic. Rust's drop glue runs on normal exits but panic does NOT guarantee Drop runs on all paths before termination. The TUI also typically enters the alternate screen buffer, which similarly is not exited on panic.

**How to avoid:**
Install a panic hook at startup before any terminal initialization:
```rust
let original_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |info| {
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(
        std::io::stderr(),
        crossterm::terminal::LeaveAlternateScreen
    );
    original_hook(info);
}));
```
Then always restore on normal exit through a struct with `impl Drop`. Do not rely on only one mechanism — implement both the panic hook and Drop-based cleanup. When restoring, suppress errors from the restore calls (use `let _ = ...`) to avoid masking the original panic reason.

**Warning signs:**
- Any code path that calls `panic!()` or unwraps a `None`/`Err` without proper error handling before terminal cleanup is wired up
- Missing panic hook in early initialization code
- Seeing `enable_raw_mode()` called without a corresponding cleanup guard struct

**Phase to address:** Phase 1 (Core TUI skeleton). Must be the first thing wired after the `Terminal::init()` call — before any application logic.

---

### Pitfall 2: Wrong Exit Code on Cancel Corrupts the Git Operation

**What goes wrong:**
When the user cancels (presses Esc or closes without saving), if the editor exits with code 0 (success), git treats the edit as successful. For a commit, git uses whatever is currently in COMMIT_EDITMSG — possibly an empty string or pre-existing content — and either aborts due to empty message or silently commits garbage. For a rebase todo file, exiting 0 with an empty or unchanged file can trigger "nothing to do" aborts or corrupt the rebase state.

**Why it happens:**
Developers implement cancel as "write nothing, exit cleanly." The correct contract with git is: exit with a **non-zero** exit code to signal that the edit was aborted. Git reads exit code 1 (or any non-zero) as "editor failed, abort the operation." Exit code 0 means "editor succeeded, proceed with the file."

**How to avoid:**
Map application states to process exit codes explicitly:
- Save and commit: write file, exit with code `0`
- Cancel / Esc: do NOT write the file, exit with code `1`
- Internal error: exit with a non-zero code and print to stderr before restoring terminal

Test this with `echo $?` after pressing Esc in a real `git commit` session before shipping.

**Warning signs:**
- `std::process::exit(0)` called unconditionally in all exit paths
- No test verifying that `git commit` is actually aborted when Esc is pressed
- Cancel described as "write empty file" rather than "exit non-zero"

**Phase to address:** Phase 1 (Core TUI skeleton / git integration contract). The exit code contract is the single most load-bearing correctness requirement of the entire application.

---

### Pitfall 3: Assuming `core.editor` Also Handles Rebase Todo Files

**What goes wrong:**
Git uses two separate editor configurations: `core.editor` for commit messages and `sequence.editor` for interactive rebase todo files (`git-rebase-todo`). If a user has set `sequence.editor` to something else (or relies on the default), `gitmedit` will never be invoked for rebase workflows even if it is set as `core.editor`. Conversely, if gitmedit is set as `sequence.editor` but not as `core.editor`, regular commits bypass it.

**Why it happens:**
The `sequence.editor` configuration was added in Git 2.40 (2023) specifically to separate rebase-todo editing from commit message editing. Most documentation and tutorials still only mention `core.editor`. Developers assume one config controls everything.

**How to avoid:**
- Installation instructions must tell users to set BOTH configs:
  ```
  git config --global core.editor gitmedit
  git config --global sequence.editor gitmedit
  ```
- The application must handle both file types (COMMIT_EDITMSG and git-rebase-todo) since it will be invoked in both contexts
- Detect context from the filename passed as `argv[1]` — do not assume one mode

**Warning signs:**
- Documentation only mentions `core.editor`
- Application code that assumes the file is always a commit message
- No test for invoking `gitmedit /path/to/git-rebase-todo`

**Phase to address:** Phase 2 (Multi-context git file handling). Must be validated before any rebase feature work.

---

### Pitfall 4: Writing Back the Rebase Todo File Corrupted or Empty

**What goes wrong:**
For interactive rebase, git reads back the exact file that was passed to the editor. If gitmedit writes an empty file (e.g., user pressed Esc but exit code was 0), writes with wrong line endings, drops the trailing newline, or garbles the `pick`/`squash`/`fixup` keywords, git either aborts with "nothing to do," proceeds with wrong operations, or enters a broken rebase state that requires `git rebase --abort` to escape.

**Why it happens:**
- Cancel-without-saving bugs (covered above) can still corrupt if the file was partially written
- Using `String::trim()` or platform line-ending normalization inadvertently changes `\n` to `\r\n` or drops a trailing newline
- Treating the rebase todo as a commit message and stripping comment lines (lines starting with `#`) — these are instructional comments that git itself strips; the editor must pass them through untouched
- Reordering or editing logic that reconstructs the file from parsed tokens instead of preserving the raw text of unchanged lines

**How to avoid:**
- Write the file atomically: write to a temp file in the same directory, then rename (avoids partial write on crash)
- Preserve all lines including comments verbatim unless the user explicitly edits them
- Use Unix line endings (`\n`) unconditionally, even on macOS
- Diff the input and output in tests: unchanged lines must be byte-for-byte identical

**Warning signs:**
- Parsing todo lines into structs and then serializing them back out (introduces reconstruction bugs)
- Using `write_all` to a file that was previously opened with truncation (`File::create`) before all content is ready
- `git rebase -i` entering a broken state during development testing

**Phase to address:** Phase 2 (Rebase / squash file handling). Every write-back path needs an integration test that does a real `git rebase -i`.

---

### Pitfall 5: Ignoring `core.commentChar` — Hardcoding `#` as the Comment Delimiter

**What goes wrong:**
Git allows users to change the comment character from `#` to any other character via `git config core.commentChar`. Some developers use `;` or another character because their commit messages contain `#issue-123` prefixes. An editor that hardcodes `#` as the comment character will either incorrectly render user-written content as comments, or fail to style actual git-generated comment lines.

Git also supports `core.commentChar = auto`, which dynamically selects a character not found at the start of any line in the existing message.

**Why it happens:**
`#` is the universally documented default. Multiple mature tools (GitExtensions, Sourcetree, old Atom) have shipped with this bug. It is only noticed by users with non-default configs.

**How to avoid:**
- Read `core.commentChar` from git config at startup via `git config --get core.commentChar`
- Fall back to `#` only if the config key is absent or returns an error
- Handle the `auto` value by scanning the file for a safe character
- Use the resolved character for both visual styling (gray/dimmed comment lines) and for the write-back logic that preserves comments

**Warning signs:**
- Comment-line detection using `line.starts_with('#')` without first reading `core.commentChar`
- No test with a non-default `core.commentChar` value

**Phase to address:** Phase 1 (Commit message handling) when comment display is first implemented. Do not defer this — retrofitting it is expensive.

---

### Pitfall 6: Alternate Screen Prevents User from Reading the `git diff` Context

**What goes wrong:**
Most TUI frameworks default to entering the alternate screen buffer (`EnterAlternateScreen`), which completely hides the user's terminal history. For a git editor, users commonly run `git commit` immediately after `git diff` or `git status`. If the editor hides that output, the user cannot reference the diff while writing the commit message. This is a significant UX regression versus nano, which does NOT use the alternate screen.

**Why it happens:**
Alternate screen is the default in ratatui's `init()` helper and in most TUI examples. Developers copy the pattern without considering that their use case (a short-lived editor, not a dashboard) has different requirements. Nano's philosophy — no alternate screen, minimal footprint — is exactly what users switching from nano expect.

**How to avoid:**
- Do NOT use `EnterAlternateScreen` / `LeaveAlternateScreen`. Render directly into the main terminal buffer.
- If some TUI layout requires full-screen control, provide an explicit opt-in configuration flag, with the default being no alternate screen.
- Test by running `git status && git commit` and verifying `git status` output is visible above the editor.

**Warning signs:**
- `ratatui::init()` used without disabling alternate screen
- `crossterm::execute!(stdout, EnterAlternateScreen)` in setup code
- No manual test of the `git diff` / `git commit` workflow in sequence

**Phase to address:** Phase 1 (TUI skeleton). Foundational layout decision — changing this later causes visible flicker and requires restructuring terminal init/restore.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Use `ratatui::init()` default (with alternate screen) | Zero boilerplate | Forces alternate-screen redesign later; breaks nano-like UX promise | Never — disable alternate screen from day one |
| Hardcode `#` as comment char | Simpler code | Bug for all users with `core.commentChar` set; noticeable regression | Only if explicitly out-of-scope for v1, documented as known limitation |
| Reconstruct todo file from parsed structs | Cleaner code model | Byte-perfect round-trip is hard; risks rebase corruption on edge cases | Never for untouched lines — preserve raw input |
| Skip panic hook, rely only on Drop | Less setup code | Terminal left in raw mode on any panic; user loses terminal session | Never |
| Single exit-code constant (always 0) | Simplest impl | Git proceeds on cancel; commits garbage or enters broken rebase | Never |
| `unwrap()` on file I/O | Fast prototyping | Panic = unrestored terminal, corrupt partial writes | Only during initial skeleton; must be eliminated before any git integration |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Git editor process model | Launching a background process that exits immediately (GUI editors need `--wait`) | gitmedit is a blocking CLI process — it MUST block until user saves or cancels, then exit. No async background mode. |
| `argv[1]` file path | Assuming the path is always relative or always absolute | The path git passes is an absolute path to a file inside `.git/`. Treat it as opaque — read it, write it, do not assume structure. |
| `sequence.editor` vs `core.editor` | Only setting `core.editor` in installation docs | Document both; handle both file types in code; test both contexts. |
| `git commit --cleanup` mode | Writing the file including comment lines when `commit.cleanup = strip` will have git strip them anyway | The editor does not need to strip comments — git does that. The editor must only preserve, not strip. |
| `core.commentChar = auto` | Treating "auto" as a literal character | Must scan the file to determine which character git would select; fall back to `#` if scanning is not implemented (document as known gap). |
| Exit code on write error | Exiting 0 after a failed file write | If the file write fails, exit non-zero. Git must not proceed with a commit based on a file that was not successfully written. |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Heavy TUI initialization before first render | Visible delay between `git commit` and editor appearing; slower than vim | Keep startup path lean: parse argv, read file, init terminal, render first frame — nothing else | Any time startup is measurably slower than `nano` on the same machine |
| Reading git config via `git config --get` subprocess per key | Multiple subprocesses add latency on every startup | Batch all needed config reads into one `git config --list` call and parse locally | Any machine where subprocess spawn overhead is noticeable (slow filesystems, containers) |
| Re-rendering the full buffer every keystroke when no content changed | CPU waste, potential flicker | Ratatui's diff-based rendering handles this — avoid forcing unconditional full redraws | Not a startup concern, but matters for responsiveness during editing |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Writing user input directly back to a path received via `argv[1]` without validation | Unlikely in normal use but `argv[1]` could be crafted in unusual environments | Verify the path exists and is a regular file before reading; refuse paths outside the repo's `.git/` directory in hardened mode |
| Logging or displaying full file path in error messages | Leaks internal repo structure in error output visible in CI logs | Keep error messages generic; do not include the full file path unless in debug mode |
| Spawning `git config` subprocess with user-controlled arguments | Shell injection if arguments are not properly separated | Use `std::process::Command` with explicit argument separation — never build shell command strings |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Alternate screen hides the `git diff` the user just ran | User cannot reference what they changed while writing the commit message | Do not use alternate screen — render into the main buffer like nano |
| No visual indication of git context (commit vs merge vs rebase) | User is confused about what they are editing | Display the file type / context in a status bar: "COMMIT", "MERGE", "REBASE (5 commits)" |
| Blocking save on empty message with no warning, then aborting | User presses Ctrl+S not realizing the message is empty, loses the edit session | Warn on save if message is empty (excluding comment lines): "Empty message — save anyway? Git will abort the commit. [Y/n]" |
| Treating all lines as editable in a rebase todo file | User accidentally edits a comment line, file is corrupted | Mark comment lines as non-editable; visually dim them; skip them in cursor navigation |
| Identical hotkeys for very different operations (Ctrl+S = commit, Esc = discard) with no confirmation | Fat-finger Esc discards a multi-line commit message with no recovery | On Esc with unsaved changes: "Discard changes? [Y/n]" prompt before exiting with code 1 |
| Showing raw rebase action keywords (`pick`, `squash`, `fixup`) without explanation | Users unfamiliar with rebase syntax do not know what to do | Show a one-line key legend at the bottom of the screen for the active mode |

---

## "Looks Done But Isn't" Checklist

- [ ] **Cancel / Esc:** Appears to work in dev testing — verify `git commit` is actually aborted (exit code is non-zero AND git prints "Aborting commit")
- [ ] **Rebase todo save:** Editor closes cleanly — verify the resulting rebase actually executed the correct operations (did not silently noop or corrupt)
- [ ] **Comment lines:** Styled as comments in the UI — verify they are written back verbatim and git still strips them correctly on commit
- [ ] **Non-default `core.commentChar`:** Editor starts — verify comment lines are correctly identified and styled when commentChar is `;` or another character
- [ ] **Panic recovery:** Application crashes mid-edit (inject a panic) — verify terminal is fully restored (raw mode off, no alternate screen remnants)
- [ ] **Multiline commit messages:** A three-paragraph message with blank lines between them — verify the blank lines are preserved in the written file and git shows them in `git log`
- [ ] **Empty message warning:** User saves with only comment lines in the buffer — verify a warning appears and git aborts only if the user confirms
- [ ] **`sequence.editor` path:** `git rebase -i HEAD~3` with `gitmedit` as `sequence.editor` — verify the todo file is opened, editable, and written back correctly

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Terminal left in raw mode after panic | LOW | User runs `reset` in their shell; no data loss |
| Git commit proceeded with garbage message after cancel | MEDIUM | `git commit --amend` to fix the message; no history loss if not pushed |
| Rebase todo file corrupted, rebase stuck | HIGH | `git rebase --abort` to restore branch; user must re-run `git rebase -i` from scratch; work is not lost but effort is |
| Alternate screen decision baked into Phase 1 — must be changed | HIGH | Requires restructuring terminal init and all render paths; ripples through the whole codebase |
| `core.commentChar` hardcoded — user reports comments not rendering | MEDIUM | Single-function fix; but shipped versions in the wild have the bug |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Terminal not restored after panic | Phase 1: TUI skeleton | Inject a `panic!()` call; verify `stty sane` is not needed afterward |
| Wrong exit code on cancel | Phase 1: git integration contract | `git commit`; press Esc; verify `git status` shows no new commit and git printed "Aborting" |
| `core.editor` vs `sequence.editor` split | Phase 2: multi-context file handling | Set only `sequence.editor`; run `git rebase -i`; verify gitmedit opens |
| Corrupt rebase todo write-back | Phase 2: rebase/squash handling | Run `git rebase -i HEAD~3` end-to-end; verify commits are reordered as expected |
| Hardcoded `#` comment char | Phase 1: commit message display | Set `git config core.commentChar ;`; run `git commit`; verify comment lines are styled correctly |
| Alternate screen hides git diff | Phase 1: TUI skeleton | Run `git diff && git commit`; verify diff output is visible above the editor |
| No context display (commit vs rebase) | Phase 2: multi-context file handling | Open each file type; verify status bar shows correct context label |
| Empty message saves without warning | Phase 1: save/cancel handling | Press Ctrl+S with empty buffer; verify warning prompt appears |

---

## Sources

- [crossterm Issue #368: Is raw mode disabled after panic?](https://github.com/crossterm-rs/crossterm/issues/368) — HIGH confidence (maintainer confirmed behavior)
- [Ratatui: Setup Panic Hooks](https://ratatui.rs/recipes/apps/panic-hooks/) — HIGH confidence (official docs)
- [Ratatui Issue #1005: Panic handler does not exit raw mode correctly](https://github.com/ratatui/ratatui/issues/1005) — HIGH confidence (official repo)
- [Git exit code behavior for aborting rebase](https://www.tutorialpedia.org/blog/how-to-abort-a-git-rebase-from-inside-vim-during-interactive-editing/) — MEDIUM confidence (community docs, consistent with git behavior)
- [Baeldung: Configure Core and Sequence Git Editors](https://www.baeldung.com/ops/git-editors-select-configure) — MEDIUM confidence (verified against git 2.40 release)
- [GitExtensions Issue #3560: Does not respect core.commentChar](https://github.com/gitextensions/gitextensions/issues/3560) — HIGH confidence (real bug report from mature tool)
- [Atom GitHub PR #1988: Respect core.commentChar](https://github.com/atom/github/pull/1988) — HIGH confidence (real fix in mature tool)
- [Magit Issue #3138: core.commentChar auto not respected](https://github.com/magit/magit/issues/3138) — HIGH confidence
- [MitMaro/git-interactive-rebase-tool Issue #739: rebase --edit-todo should not empty file](https://github.com/MitMaro/git-interactive-rebase-tool/issues/739) — HIGH confidence (domain expert project)
- [VSCode Issue: empty git-rebase-todo on rebase -i](https://github.com/microsoft/vscode/issues/92628) — HIGH confidence (widely reproduced)
- [Lazygit 5 Years On](https://jesseduffield.com/Lazygit-5-Years-On/) — MEDIUM confidence (author retrospective)
- [HN: Show HN: I made a git rebase TUI editor](https://news.ycombinator.com/item?id=41831073) — MEDIUM confidence (community discussion)
- [unicode-width crate documentation](https://github.com/unicode-rs/unicode-width) — HIGH confidence (official crate)

---

*Pitfalls research for: TUI git editor (Rust, core.editor replacement)*
*Researched: 2026-03-18*
