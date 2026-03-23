# Phase 5: Installation + Distribution - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can install gitmedit with a single `cargo install gitmedit` command (from crates.io), find it in PATH at `~/.cargo/bin/gitmedit`, and manually configure git to use it for both commits (`core.editor`) and interactive rebase (`sequence.editor`).

</domain>

<decisions>
## Implementation Decisions

### Release Strategy
- Publish to crates.io as a public crate for discoverability
- Initial version: v0.1.0 (signals early/stable status, allows future iteration)
- Users install via: `cargo install gitmedit`
- Also support local: `cargo install --path crates/gitmedit` from repo

### Configuration
- No automatic setup script in Phase 5
- Document manual configuration steps in README:
  - `git config --global core.editor gitmedit` (for commits)
  - `git config --global sequence.editor gitmedit` (for interactive rebase)
- Users run these commands manually after installation
- Future phase (v2) can add `--setup` flag if needed

### Platform Support
- v1.0 supports: Linux, macOS, **and Windows**
- Use crossterm features that work on all three platforms
- Test on Windows terminal (not WSL, actual Windows)
- No platform-specific code paths in v1 (keep simple)

### Error Handling & Messages
- Clear, actionable error messages when installation fails:
  - If PATH not writable → explain and suggest alternatives
  - If git not found → suggest installing git first
  - If permission denied → explain sudo risks, suggest alternatives
  - If dependencies missing → clear next steps
- Errors exit with code 1 and message to stderr
- Success: binary installed to `~/.cargo/bin/gitmedit`, exit 0

### Build & Package Metadata
- Cargo.toml: Add description, homepage, repository, license fields for crates.io
- Include README with installation + git config instructions
- MIT or Apache 2.0 license (standard for Rust)

### Claude's Discretion
- Exact error message wording and formatting
- How to detect PATH location after installation
- Cross-platform binary name handling (gitmedit.exe on Windows vs gitmedit on Unix)
- Optional: shell completion setup instructions

</decisions>

<canonical_refs>
## Canonical References

### Requirements
- `.planning/REQUIREMENTS.md` INSTALL-01 through INSTALL-05 — Installation, PATH availability, git config

### Architecture & Stack
- `.planning/research/STACK.md` — Technology choices (ratatui 0.30, crossterm 0.29, crossterm platform support matrix)
- `.planning/ROADMAP.md` Phase 5 — Phase goal, success criteria, dependencies

### Prior Phases
- `.planning/phases/01-git-contract-tui-shell/01-CONTEXT.md` — Exit code contract, context detection, atomicity patterns
- All prior phases — No external installation logic needed, Phase 5 handles distribution

</canonical_refs>

<code_context>
## Existing Code Insights

### Cargo Project Structure
- Root Cargo.toml: workspace with `members = ["crates/gitmedit"]`
- crates/gitmedit/Cargo.toml: primary binary crate
- Phase 5 updates: Add metadata (description, repository, license) to crates/gitmedit/Cargo.toml

### Binary Output
- Cargo build output: `target/debug/gitmedit` (debug) or `target/release/gitmedit` (release)
- cargo install handles --path and crates.io paths automatically
- No custom build.rs needed for v1

### Integration Points
- Entry point: src/main.rs (already exists, no changes needed)
- Exit code contract: main.rs returns exit codes 0/1 to git (already implemented)
- No runtime configuration files — sensible defaults only (per PROJECT.md)

</code_context>

<specifics>
## Specific Ideas

- Installation should be one command: `cargo install gitmedit`
- After installation, git config should be straightforward — one command per editor type
- Windows support ensures tool works for all git users, not just Unix developers
- Error messages should help users debug (clear, not cryptic)

</specifics>

<deferred>
## Deferred Ideas

- `gitmedit --setup` automatic configuration script — v2 feature
- Shell completion setup (bash, zsh, fish) — v2 feature
- Homebrew/apt package distribution — v2 feature
- Version checking / auto-update — v2+ feature

</deferred>

---

*Phase: 05-installation-distribution*
*Context gathered: 2026-03-23*
