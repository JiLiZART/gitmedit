# Phase 5: Installation + Distribution - Research

**Researched:** 2026-03-24
**Domain:** Rust crate publishing (crates.io), cargo install, git editor configuration
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Release Strategy**
- Publish to crates.io as a public crate for discoverability
- Initial version: v0.1.0 (signals early/stable status, allows future iteration)
- Users install via: `cargo install gitmedit`
- Also support local: `cargo install --path crates/gitmedit` from repo

**Configuration**
- No automatic setup script in Phase 5
- Document manual configuration steps in README:
  - `git config --global core.editor gitmedit` (for commits)
  - `git config --global sequence.editor gitmedit` (for interactive rebase)
- Users run these commands manually after installation
- Future phase (v2) can add `--setup` flag if needed

**Platform Support**
- v1.0 supports: Linux, macOS, and Windows
- Use crossterm features that work on all three platforms
- Test on Windows terminal (not WSL, actual Windows)
- No platform-specific code paths in v1 (keep simple)

**Error Handling & Messages**
- Clear, actionable error messages when installation fails:
  - If PATH not writable → explain and suggest alternatives
  - If git not found → suggest installing git first
  - If permission denied → explain sudo risks, suggest alternatives
  - If dependencies missing → clear next steps
- Errors exit with code 1 and message to stderr
- Success: binary installed to `~/.cargo/bin/gitmedit`, exit 0

**Build & Package Metadata**
- Cargo.toml: Add description, homepage, repository, license fields for crates.io
- Include README with installation + git config instructions
- MIT or Apache 2.0 license (standard for Rust)

### Claude's Discretion
- Exact error message wording and formatting
- How to detect PATH location after installation
- Cross-platform binary name handling (gitmedit.exe on Windows vs gitmedit on Unix)
- Optional: shell completion setup instructions

### Deferred Ideas (OUT OF SCOPE)
- `gitmedit --setup` automatic configuration script — v2 feature
- Shell completion setup (bash, zsh, fish) — v2 feature
- Homebrew/apt package distribution — v2 feature
- Version checking / auto-update — v2+ feature
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| INSTALL-01 | Binary can be installed with `cargo install --path crates/gitmedit` | cargo install --path works from repo root; installs to ~/.cargo/bin/ automatically |
| INSTALL-02 | Binary is available in PATH after installation | cargo install places binary in ~/.cargo/bin/; cargo itself warns users if that dir is not in PATH |
| INSTALL-03 | User can set as `core.editor` with `git config --global core.editor gitmedit` | git invokes editor as `<command> <filepath>`; simple binary name works when binary is in PATH |
| INSTALL-04 | User can set as `sequence.editor` with `git config --global sequence.editor gitmedit` | git passes git-rebase-todo path to sequence.editor; same invocation contract as core.editor |
| INSTALL-05 | Editor respects both `core.editor` and `sequence.editor` configs | ALREADY IMPLEMENTED: context.rs detect_context() dispatches on filename (COMMIT_EDITMSG → Commit, git-rebase-todo → Rebase); no code change needed |
</phase_requirements>

---

## Summary

Phase 5 is primarily a packaging and documentation phase. The editor binary is fully functional; this phase adds the Cargo.toml metadata required for crates.io publication, creates the README, and verifies the end-to-end install flow works.

**Key insight:** INSTALL-05 is already satisfied. The `detect_context()` function in `src/context.rs` dispatches on the filename passed by git — COMMIT_EDITMSG (core.editor invocations) and git-rebase-todo (sequence.editor invocations) already resolve to correct modes. No code changes are needed for INSTALL-05.

**Crate name availability confirmed:** `cargo install gitmedit` will work — the name `gitmedit` does not exist on crates.io (verified via `https://crates.io/api/v1/crates/gitmedit` returning `{"errors":[{"detail":"crate \`gitmedit\` does not exist"}]}`).

**Primary recommendation:** Add 5 metadata fields to Cargo.toml (description, license, homepage, repository, readme), create a README.md, verify `cargo publish --dry-run` passes, then publish. The install/config verification steps confirm INSTALL-01 through INSTALL-04 are satisfied.

---

## Standard Stack

### Core (no new dependencies needed)

| Component | Current State | Phase 5 Change |
|-----------|--------------|----------------|
| Cargo.toml | Has `name`, `version`, `edition`, `dependencies` | Add `description`, `license`, `repository`, `homepage`, `readme` |
| src/context.rs | Fully implemented detect_context() | No changes |
| Binary output | `target/release/gitmedit` | No changes |
| cargo install | Works with `--path` | Verify also works from crates.io |

No new crate dependencies are required for this phase. This is a metadata + documentation phase.

### Cargo.toml Metadata Fields Required by crates.io

| Field | Required | Notes |
|-------|----------|-------|
| `description` | YES (by crates.io) | Short plain text, 1-2 sentences, no Markdown |
| `license` | YES (by crates.io) | SPDX expression, e.g. `"MIT OR Apache-2.0"` |
| `repository` | No (but strongly recommended) | GitHub URL |
| `homepage` | No (optional) | Only if dedicated website exists; omit if same as repository |
| `readme` | No (auto-detected if README.md exists) | Explicit path preferred |
| `keywords` | No | Max 5, ASCII, max 20 chars each |
| `categories` | No | Max 5, must match crates.io official slugs |

**Source:** Official Cargo manifest docs — [https://doc.rust-lang.org/cargo/reference/manifest.html](https://doc.rust-lang.org/cargo/reference/manifest.html)

### Recommended Cargo.toml After Phase 5

```toml
[package]
name = "gitmedit"
version = "0.1.0"
edition = "2024"
description = "Fast, distraction-free git editor with nano-style shortcuts and context-aware modes for commits, merges, and interactive rebase."
license = "MIT OR Apache-2.0"
repository = "https://github.com/jilizart/gitmedit"
readme = "README.md"
keywords = ["git", "editor", "tui", "commit", "rebase"]
categories = ["command-line-utilities", "development-tools"]

[dependencies]
# ... unchanged
```

**Note on `homepage`:** Per official docs, `homepage` should only be set if there is a dedicated website separate from the repository. Since there is no dedicated website, omit it. Setting it to the same URL as `repository` is considered redundant.

---

## Architecture Patterns

### How git invokes the editor (HIGH confidence)

Git invokes both `core.editor` and `sequence.editor` with a single positional argument: the path to the file to edit. The binary is run as a blocking subprocess; git waits for exit code 0 (save) or non-zero (abort).

```
# core.editor invocation (commit messages):
gitmedit /path/to/.git/COMMIT_EDITMSG

# sequence.editor invocation (interactive rebase):
gitmedit /path/to/.git/rebase-merge/git-rebase-todo
```

Git's editor lookup order:
1. `GIT_EDITOR` environment variable
2. `core.editor` config value
3. `VISUAL` environment variable
4. `EDITOR` environment variable
5. Compile-time default (usually `vi`)

For `sequence.editor`:
1. `GIT_SEQUENCE_EDITOR` environment variable
2. `sequence.editor` config value
3. Falls back to `GIT_EDITOR` (same chain as above)

**Why this satisfies INSTALL-05 without code changes:** gitmedit already detects mode from the filename, not from which config key invoked it. `git-rebase-todo` → Rebase mode. `COMMIT_EDITMSG` → Commit mode. The same binary handles both correctly regardless of whether it was invoked via `core.editor` or `sequence.editor`.

### cargo install PATH mechanics (HIGH confidence)

```
# Default install root resolution order:
1. --root flag
2. $CARGO_INSTALL_ROOT env var
3. install.root in ~/.cargo/config.toml
4. $CARGO_HOME env var
5. ~/.cargo (default)

# Binary ends up at:
~/.cargo/bin/gitmedit          # Unix/macOS
%USERPROFILE%\.cargo\bin\gitmedit.exe   # Windows
```

Cargo automatically warns users at install time if `~/.cargo/bin` is not in PATH:

```
warning: be sure to add `~/.cargo/bin` to your PATH to be able to run the installed binaries
```

**Source:** [https://doc.rust-lang.org/cargo/commands/cargo-install.html](https://doc.rust-lang.org/cargo/commands/cargo-install.html)

### Windows binary name handling (HIGH confidence)

Cargo automatically appends `.exe` on Windows. When users set:
```
git config --global core.editor gitmedit
```
Git for Windows resolves `gitmedit` to `gitmedit.exe` when searching PATH — no special handling needed in the config command. The user uses the same command on all platforms.

### Publishing workflow

```bash
# 1. One-time: create account at crates.io, generate API token
cargo login   # prompts for token, stores in ~/.cargo/credentials.toml

# 2. Verify package before uploading (no network upload)
cargo publish --dry-run

# 3. Check what files will be included
cargo package --list

# 4. Publish
cargo publish
```

**Permanence:** Published versions cannot be deleted or overwritten. Use `cargo yank --version X.Y.Z` to prevent new installations of a broken release without removing it.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| PATH detection after install | Custom shell script or Rust code to probe PATH | Cargo itself warns users | Cargo's install already prints PATH warning if `~/.cargo/bin` is missing from PATH |
| Windows .exe extension handling | Conditional compile or runtime check | Cargo's native cross-platform binary naming | Cargo automatically appends .exe on Windows; git for Windows resolves it |
| License text files | Custom text | SPDX expression in `license` field | crates.io accepts `"MIT OR Apache-2.0"` with no file needed; standard license text is publicly known |
| crates.io publishing script | bash/python publish automation | `cargo publish` | cargo publish handles packaging, verification, and upload atomically |

---

## Common Pitfalls

### Pitfall 1: Missing crates.io-required fields
**What goes wrong:** `cargo publish` fails with "missing field: description" or "missing field: license"
**Why it happens:** Fields that are optional for local builds are required by the crates.io registry
**How to avoid:** Add all required fields before attempting publish; run `cargo publish --dry-run` to catch errors locally
**Warning signs:** Any `cargo publish --dry-run` error mentioning "field" or "metadata"

### Pitfall 2: README path mismatch
**What goes wrong:** crates.io page shows no README; or `cargo publish` fails because readme path doesn't resolve
**Why it happens:** The `readme` field in Cargo.toml must be a path relative to Cargo.toml; if the file doesn't exist, publish fails
**How to avoid:** Create README.md at the project root; set `readme = "README.md"` in Cargo.toml; verify with `cargo package --list` that README.md appears
**Warning signs:** `cargo package --list` output doesn't include README.md

### Pitfall 3: Cargo.toml version already published
**What goes wrong:** `cargo publish` fails with "crate version already uploaded"
**Why it happens:** crates.io does not allow re-publishing the same version number
**How to avoid:** For the first publish, use a version that hasn't been published (0.1.0 is safe since crate doesn't exist yet)
**Warning signs:** Any publish attempt after a previous successful publish without bumping the version

### Pitfall 4: INSTALL-01 path is wrong for the current project structure
**What goes wrong:** `cargo install --path crates/gitmedit` fails because the project is NOT a workspace
**Why it happens:** The current Cargo.toml is at the project root `./Cargo.toml` with `name = "gitmedit"`, not in a `crates/gitmedit/` subdirectory. CONTEXT.md references a workspace layout that was planned but not executed.
**How to avoid:** The correct local install command is `cargo install --path .` from the project root. REQUIREMENTS.md INSTALL-01 says "cargo install --path crates/gitmedit" which implies a workspace. **This needs verification during planning** — either update the path in INSTALL-01 to match current structure, or create the workspace layout.
**Warning signs:** "no Cargo.toml found in crates/gitmedit"

> Note on workspace: STATE.md records "[Phase 01-01]: Single-crate package not workspace - Phase 5 adds workspace split per plan." This means the workspace migration was explicitly deferred to Phase 5. Planning should decide: (a) create the `crates/gitmedit/` workspace layout as originally planned, or (b) use `cargo install --path .` and update INSTALL-01 wording. Both are valid; the planner should make this explicit.

### Pitfall 5: Forgetting that LICENSE file should exist on disk
**What goes wrong:** Using `license = "MIT OR Apache-2.0"` in Cargo.toml but having no LICENSE file in the repository — crates.io accepts it, but GitHub and users expect an actual file
**Why it happens:** The SPDX field satisfies crates.io's requirement, but convention dictates a LICENSE file in the repo
**How to avoid:** Create a LICENSE file (MIT and Apache-2.0 standard texts are publicly available) or at minimum a LICENSE-MIT file
**Warning signs:** No LICENSE file shown in `cargo package --list`

### Pitfall 6: categories must match crates.io official slugs exactly
**What goes wrong:** `cargo publish` fails with "invalid category slug"
**Why it happens:** Categories are validated against the official list at https://crates.io/category_slugs — free-form text is rejected
**How to avoid:** Use slugs from the official list: `"command-line-utilities"` and `"development-tools"` are both valid
**Warning signs:** Any category publish error mentioning "invalid" or "not found"

---

## Code Examples

### Minimal crates.io-ready Cargo.toml

```toml
# Source: https://doc.rust-lang.org/cargo/reference/manifest.html
[package]
name = "gitmedit"
version = "0.1.0"
edition = "2024"
description = "Fast, distraction-free git editor with context-aware modes for commits, merges, and interactive rebase."
license = "MIT OR Apache-2.0"
repository = "https://github.com/OWNER/gitmedit"
readme = "README.md"
keywords = ["git", "editor", "tui", "commit", "rebase"]
categories = ["command-line-utilities", "development-tools"]
```

### Verify package contents before publish

```bash
# Source: https://doc.rust-lang.org/cargo/commands/cargo-install.html
cargo package --list        # list files that will be uploaded
cargo publish --dry-run     # full verification without upload
```

### Install and verify locally

```bash
# Install from local path (adjust path to match actual structure)
cargo install --path .

# Verify binary is installed
cargo install --list | grep gitmedit

# Verify PATH availability
which gitmedit         # Unix/macOS
where gitmedit         # Windows
```

### Configure git to use gitmedit

```bash
# For commit messages (core.editor)
git config --global core.editor gitmedit

# For interactive rebase (sequence.editor)
git config --global sequence.editor gitmedit

# Verify
git config --global --get core.editor
git config --global --get sequence.editor
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual git editor shell wrappers | Single binary via `cargo install` | N/A | Simpler UX; no wrapper scripts |
| Separate tools for commit vs rebase | Single binary handles both via filename detection | N/A | Single install command covers both use cases |
| `cargo-dist` for cross-platform releases | `cargo install` sufficient for v1 | N/A (cargo-dist is a v2+ option) | Lower complexity for initial release |

**Relevant current information:**
- crates.io `gitmedit` name is unclaimed (verified 2026-03-24 via API)
- `cargo publish` requires `description` and `license` — confirmed for current Cargo 1.x
- `cargo install` auto-appends `.exe` on Windows — stable behavior

---

## Open Questions

1. **Workspace structure: create crates/gitmedit/ or update INSTALL-01?**
   - What we know: Current project is single-crate at root. STATE.md says workspace split was planned for Phase 5. CONTEXT.md references `cargo install --path crates/gitmedit`. INSTALL-01 in REQUIREMENTS.md uses the `crates/gitmedit` path.
   - What's unclear: Whether the workspace migration is actually needed for Phase 5's goals, or whether it's unnecessary complexity that was deferred from Phase 1 "by default."
   - Recommendation: The planner should make an explicit decision. Creating a workspace for a single-crate project adds structural overhead with no current benefit — the simplest path is `cargo install --path .` and updating INSTALL-01 wording. However, if future phases will add more crates (e.g., a gitmedit-core library), the workspace structure makes sense now.

2. **Homepage field: include or omit?**
   - What we know: There is no dedicated website. Official docs say homepage should only be set if there is a dedicated website separate from repository.
   - What's unclear: Whether pointing homepage to the GitHub repo page is acceptable or frowned upon.
   - Recommendation: Omit `homepage` entirely. Setting it identical to `repository` is redundant and the crates.io docs explicitly say not to do this.

3. **LICENSE file: MIT, Apache-2.0, or dual?**
   - What we know: CONTEXT.md says "MIT or Apache 2.0 (standard for Rust)." The Rust ecosystem standard is dual-licensing as `"MIT OR Apache-2.0"`.
   - What's unclear: Whether to include one LICENSE file or two (LICENSE-MIT and LICENSE-APACHE).
   - Recommendation (Claude's discretion): Use `license = "MIT OR Apache-2.0"` in Cargo.toml and create a single `LICENSE` file containing the MIT license text, which is the most commonly used of the two. Alternatively, create both `LICENSE-MIT` and `LICENSE-APACHE`. Either approach is accepted by crates.io.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| cargo | INSTALL-01, crates.io publish | Yes | 1.92.0-nightly | — |
| rustc | Build | Yes | 1.92.0-nightly | — |
| crates.io account + API token | cargo publish | Must be set up (one-time) | n/a | cargo publish --dry-run works without token |
| git | Verify INSTALL-03/04 | Assumed present | — | — |

**Missing dependencies with no fallback:**
- crates.io API token: required for `cargo publish` (not `--dry-run`). Must be created at https://crates.io/settings/tokens and registered via `cargo login`.

**Missing dependencies with fallback:**
- None for local install verification (INSTALL-01, INSTALL-02 can be verified entirely locally without crates.io access).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test (`#[test]`) |
| Config file | none (cargo built-in) |
| Quick run command | `cargo test` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| INSTALL-01 | `cargo install --path .` completes without error | smoke | `cargo install --path . && echo OK` | n/a (shell command) |
| INSTALL-02 | Binary available at `~/.cargo/bin/gitmedit` | smoke | `which gitmedit` or `cargo install --list \| grep gitmedit` | n/a (shell command) |
| INSTALL-03 | git invokes gitmedit for commit messages | manual integration | `git config --global core.editor gitmedit && git commit` | n/a (requires git repo) |
| INSTALL-04 | git invokes gitmedit for interactive rebase | manual integration | `git config --global sequence.editor gitmedit && git rebase -i HEAD~2` | n/a (requires git repo with commits) |
| INSTALL-05 | context.rs correctly dispatches Commit vs Rebase by filename | unit (existing) | `cargo test context` | YES (src/context.rs tests) |

### Sampling Rate
- **Per task commit:** `cargo test` (86 existing tests, completes in ~0.1s)
- **Per wave merge:** `cargo test && cargo publish --dry-run`
- **Phase gate:** `cargo publish --dry-run` green + manual install smoke test before `/gsd:verify-work`

### Wave 0 Gaps
None — existing test infrastructure covers all automated requirements. INSTALL-01 through INSTALL-04 require smoke/manual validation steps, not new test files.

---

## Sources

### Primary (HIGH confidence)
- [https://doc.rust-lang.org/cargo/reference/manifest.html](https://doc.rust-lang.org/cargo/reference/manifest.html) — Cargo.toml metadata fields: description, license, readme, keywords, categories
- [https://doc.rust-lang.org/cargo/reference/publishing.html](https://doc.rust-lang.org/cargo/reference/publishing.html) — crates.io publish workflow, dry-run, required fields
- [https://doc.rust-lang.org/cargo/commands/cargo-install.html](https://doc.rust-lang.org/cargo/commands/cargo-install.html) — cargo install PATH mechanics, --path flag, cross-platform binary naming
- [https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html) — complete publish workflow, yanking, version permanence
- crates.io API: `https://crates.io/api/v1/crates/gitmedit` returns 404 — name is available (verified 2026-03-24)
- `.planning/research/STACK.md` — existing stack decisions (ratatui 0.30, crossterm 0.29, cargo release profile)
- `src/context.rs` — detect_context() implementation confirms INSTALL-05 is already satisfied

### Secondary (MEDIUM confidence)
- [https://www.baeldung.com/ops/git-editors-select-configure](https://www.baeldung.com/ops/git-editors-select-configure) — git editor invocation order (GIT_EDITOR → core.editor → VISUAL → EDITOR)
- [https://git-scm.com/docs/git-var](https://git-scm.com/docs/git-var) — GIT_SEQUENCE_EDITOR vs GIT_EDITOR lookup chain

### Tertiary (LOW confidence — not used for locked decisions)
- WebSearch results on Windows git editor configuration — general pattern confirmed by primary sources above

---

## Metadata

**Confidence breakdown:**
- Cargo.toml metadata requirements: HIGH — verified against official Cargo Book
- cargo install PATH mechanics: HIGH — verified against official cargo-install docs
- crates.io name availability: HIGH — verified via live API call
- INSTALL-05 already satisfied: HIGH — verified by reading src/context.rs source code
- Workspace structure decision: MEDIUM — STATE.md records intent but no implementation; open question for planner
- git editor invocation contract: MEDIUM — confirmed by multiple sources, exact shell quoting behavior varies by OS/shell

**Research date:** 2026-03-24
**Valid until:** 2026-06-24 (stable Cargo publishing API; crates.io policies rarely change)
