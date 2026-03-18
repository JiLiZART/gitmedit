# Feature Research

**Domain:** TUI git commit/rebase editor (replaces core.editor)
**Researched:** 2026-03-18
**Confidence:** HIGH for table stakes (well-understood git contracts); MEDIUM for differentiators (fewer prior art examples in this narrow niche)

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels broken or unshippable.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Open file passed as argv[1] | Git passes the message file path as the first argument; if the editor ignores it, nothing works | LOW | Must read COMMIT_EDITMSG, MERGE_MSG, git-rebase-todo, SQUASH_MSG, etc. |
| Display file content in editable text area | Core editing capability | LOW | Full buffer display with cursor |
| Save and exit (success path) | User must be able to commit | LOW | Write modified buffer back to the file, exit 0 |
| Cancel/abort without committing | User must be able to back out | LOW | Exit non-zero OR clear the file — git treats empty message as abort |
| Preserve comment lines (lines starting with `#`) | Git uses comments to provide context to the user; they are stripped by git, not the editor | LOW | Comments must round-trip unchanged; ideally styled differently so they feel non-editable |
| Handle multiline body text | Standard commit messages have subject + blank line + body; users write paragraphs | MEDIUM | Correct newline handling, scrolling if content exceeds viewport |
| Correct exit code contract | Git relies on exit codes: 0 = proceed, non-zero = abort | LOW | Critical correctness requirement; wrong exit codes corrupt workflows |
| Fast startup (sub-200ms) | The entire reason users replace vim/nano; if slow, there's no value | LOW | Rust binary with no runtime; startup is near-instant by default |
| Handle rebase-todo file format | git interactive rebase passes git-rebase-todo; editors that can't parse it are useless for rebase | HIGH | Format: `pick|squash|fixup|reword|edit|drop|exec|break <hash> <message>` |
| Display keybinding hints | Nano popularized always-visible shortcuts; users coming from nano expect this | LOW | Bottom bar showing Ctrl+S / Esc at minimum |
| Work as git sequence editor | Git uses GIT_SEQUENCE_EDITOR for rebase todo; must work in both roles (core.editor and sequence editor) | MEDIUM | Same binary, different file format; detect by file contents |

### Differentiators (Competitive Advantage)

Features that set gitmedit apart. Not required, but aligned with the "git-aware" value proposition.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Git context detection | Automatically detects which scenario it's in (standard commit, merge, squash, rebase) and adjusts UI accordingly | MEDIUM | Detect by filename (COMMIT_EDITMSG vs MERGE_MSG vs git-rebase-todo) and by content (squash comments in COMMIT_EDITMSG) |
| Subject line character counter (50/72 rule) | Live feedback prevents bad commits; 50-char subject limit is a strong convention enforced by GitHub; no plain editor shows this | LOW | Color-change at 51+ chars on subject line; second threshold at 73+ |
| Rebase action manipulation via hotkeys | git-interactive-rebase-tool is the reference here, but it's a separate install; gitmedit could cover this natively | HIGH | p/pick, s/squash, f/fixup, d/drop, r/reword via single keypress on line; reordering is harder |
| Visual distinction for commit log in squash context | When squashing, COMMIT_EDITMSG includes the accumulated log of all squashed commits; rendering this as read-only/styled context helps users write better combined messages | MEDIUM | Detect the squash comment block, render it styled (dim/italic), cursor stays out of it |
| Comment block styling | Render `#`-prefixed lines with distinct color (dim grey) so users never accidentally edit them | LOW | Pure rendering concern; high signal-to-noise improvement |
| Line-length ruler at column 72 | Visual marker in the body text area shows the 72-character soft wrap point | LOW | A vertical bar or color shift at column 72 |

### Anti-Features (Commonly Requested, Often Problematic)

Features to deliberately exclude, with rationale.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Vim/Emacs keybindings | Power users want their muscle memory | Adds config surface area, makes "simple" claim hollow, alienates the target audience (people who don't want vim) | Document that vim users should keep using vim; gitmedit targets the "not vim" crowd |
| Plugin system / extensibility | Developers always want hooks | Violates the single-purpose tool philosophy; plugins cause startup overhead and maintenance burden | Keep behavior fixed; if users need extensibility, lazygit or fugitive is the right tool |
| Syntax highlighting of code | Seems useful for commit bodies | Commit messages aren't code; highlighting adds complexity with near-zero utility | Apply styling only to structural git elements (comments, action words in rebase-todo) |
| Configuration file | Users want to customize colors, keys | Config files add complexity, docs burden, and edge cases; the tool should work without any config | Sensible fixed defaults; no config file for v1 |
| GUI / graphical window | Some users ask for popups or mouse support | This is a terminal editor; mouse support in terminals is fragile across environments | Full keyboard-driven; no mouse dependency |
| Spell checking | Users want help writing good messages | Requires a dictionary dependency; dramatically increases binary size and startup | Out of scope; users write commits in their language |
| Commit template loading | Git supports commit.template config | Complex path resolution, user config parsing; adds failure modes | v1 displays whatever git passes in the file — template content arrives pre-inserted by git itself |
| History of previous messages | Autocomplete from past commits | Requires persistent storage, git log parsing; adds complexity disproportionate to benefit | Users can use shell history (up-arrow) or lazygit for advanced workflows |

---

## Feature Dependencies

```
[File path argv parsing]
    └──requires──> [File content loading]
                       └──requires──> [Editable text area]
                                          └──requires──> [Save / exit 0]
                                          └──requires──> [Cancel / exit non-zero]

[Git context detection]
    └──requires──> [File path argv parsing]
    └──enhances──> [Comment block styling]
    └──enhances──> [Squash context rendering]
    └──enhances──> [Rebase action manipulation]

[Subject line character counter]
    └──requires──> [Editable text area]
    └──requires──> [Git context detection] (only active in COMMIT_EDITMSG, not rebase-todo)

[Rebase action manipulation]
    └──requires──> [Git context detection] (must know we're in rebase-todo mode)
    └──requires──> [Editable text area]
    └──conflicts──> [Free-form editing] (in rebase-todo, lines have structured format; freeform editing breaks it)

[Comment block styling]
    └──requires──> [Editable text area]
    └──enhances──> [Squash context rendering]

[Squash context rendering]
    └──requires──> [Git context detection]
    └──requires──> [Comment block styling]
```

### Dependency Notes

- **File path parsing requires file content loading:** The editor is useless without reading the file git passes.
- **Git context detection enhances multiple features:** Context detection is the multiplier — it unlocks squash rendering, rebase actions, and context-aware subject line counter. Build it early.
- **Rebase action manipulation conflicts with free-form editing in todo files:** The git-rebase-todo format is structured. Allowing arbitrary text editing of action words risks producing invalid todo files. The hotkey approach (cycle through pick/squash/fixup/drop) prevents malformed output.
- **Subject line counter requires context detection:** The counter is only meaningful in commit message files, not in rebase-todo files where there is no subject line.

---

## MVP Definition

### Launch With (v1)

Minimum viable product — validates the "fast, git-aware editor" concept.

- [ ] Open file from argv[1] and display contents in editable buffer — without this, nothing works
- [ ] Basic text editing: type, delete, arrow navigation, newlines — fundamental editing capability
- [ ] Save with Ctrl+S (exit 0) and cancel with Esc (exit non-zero) — satisfies the git editor contract
- [ ] Comment lines (`#`) rendered visually distinct and preserved on save — prevents user confusion, preserves git's context system
- [ ] Visible keybinding bar at bottom — required for discoverability; nano proved this is table stakes
- [ ] Handle COMMIT_EDITMSG (standard commits) and MERGE_MSG (merge commits) — these are the two most common invocation paths
- [ ] Handle git-rebase-todo (rebase sequence editor) with action display — must work for interactive rebase
- [ ] Correct exit codes per git expectations — correctness; wrong codes = data loss risk

### Add After Validation (v1.x)

Features to add once the core editing loop is working and dogfooding reveals real friction.

- [ ] Subject line character counter with 50/72 color coding — small implementation cost, high daily utility for users who care about commit hygiene
- [ ] Rebase action cycling with hotkeys (p/s/f/d/r) — once rebase-todo display works, action editing is the next friction point
- [ ] Squash context rendering (accumulated commit log as styled read-only block) — reduces confusion during squash merges
- [ ] Line-length ruler at column 72 in commit body — low-cost quality-of-life improvement

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] Reorder rebase-todo lines — complex interaction model (visual selection + move); defer until basic action manipulation is validated
- [ ] Undo/redo history — useful but adds state management complexity; basic editing without undo is acceptable for a commit editor used in short sessions
- [ ] Mouse support — fragile across terminal emulators; adds complexity; keyboard-only is the right default

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Open file + display buffer | HIGH | LOW | P1 |
| Basic text editing | HIGH | LOW | P1 |
| Save (Ctrl+S) / Cancel (Esc) with correct exit codes | HIGH | LOW | P1 |
| Comment line styling | HIGH | LOW | P1 |
| Keybinding bar | HIGH | LOW | P1 |
| COMMIT_EDITMSG + MERGE_MSG handling | HIGH | LOW | P1 |
| Rebase-todo display | HIGH | MEDIUM | P1 |
| Git context detection (file type sniffing) | HIGH | LOW | P1 |
| Subject line character counter | MEDIUM | LOW | P2 |
| Rebase action hotkeys (cycle pick/squash/fixup/drop) | HIGH | MEDIUM | P2 |
| Squash context rendering | MEDIUM | MEDIUM | P2 |
| Line-length ruler (column 72) | LOW | LOW | P2 |
| Rebase-todo line reordering | MEDIUM | HIGH | P3 |
| Undo/redo | MEDIUM | HIGH | P3 |
| Mouse support | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

---

## Competitor Feature Analysis

The direct competitor category is "tools that can serve as git core.editor." No competitor targets this narrow niche with a dedicated TUI. gitmedit is novel in that space.

| Feature | vim/nano (default editors) | git-interactive-rebase-tool | lazygit (embedded editor) | gitmedit target |
|---------|---------------------------|----------------------------|--------------------------|-----------------|
| Fast startup | nano: fast; vim: fast | Fast (Rust) | N/A — it's a full TUI | Fast (Rust, ~50ms) |
| Git context awareness | None — dumb text editors | Rebase-only, no commit editing | Full but separate workflow | All git editor contexts |
| Comment line styling | None | N/A | N/A | Yes — dim/styled |
| Subject line counter | None (vim plugins only) | N/A | None | Yes (v1.x) |
| Rebase action manipulation | None — manual typing | Yes — full featured | Yes via UI | Basic hotkeys (v1.x) |
| Keybinding discoverability | Poor (vim) / OK (nano) | `?` help key | Yes | Always-visible bar |
| Works as both core.editor and sequence.editor | Only dumb text editing | Sequence editor only | Neither | Both roles |
| Installation | Pre-installed | Separate install | Separate install | Single binary, set once |
| Minimal surface area | vim: large; nano: medium | Medium | Very large | Minimal |

---

## Sources

- [git-interactive-rebase-tool](https://github.com/MitMaro/git-interactive-rebase-tool) — reference for rebase TUI feature set (HIGH confidence, official GitHub)
- [Git rebase documentation](https://git-scm.com/docs/git-rebase) — rebase-todo file format specification (HIGH confidence, official docs)
- [Git commit documentation](https://git-scm.com/docs/git-commit) — COMMIT_EDITMSG behavior, exit code expectations (HIGH confidence, official docs)
- [The 50/72 rule](https://cbea.ms/git-commit/) — subject line and body character conventions (HIGH confidence, widely cited reference)
- [lazygit features](https://github.com/jesseduffield/lazygit) — what a full TUI git client covers (HIGH confidence, official GitHub)
- [gitui features](https://github.com/gitui-org/gitui) — Rust TUI git client comparison (HIGH confidence, official GitHub)
- [Baeldung: Git editors](https://www.baeldung.com/ops/git-editors-select-configure) — core.editor vs sequence editor distinction (MEDIUM confidence, tutorial site verified against git docs)
- WebSearch: git editor startup latency, user expectations — patterns consistent across multiple sources (MEDIUM confidence)

---

*Feature research for: TUI git commit/rebase editor*
*Researched: 2026-03-18*
