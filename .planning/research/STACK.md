# Technology Stack

**Project:** gitmedit v1.1 — Standalone Commit + Editor Overhaul
**Researched:** 2026-04-07
**Milestone context:** Adds to existing ratatui 0.30 / crossterm 0.29 / clap 4.6 / arboard / ratatui-textarea 0.8 base

---

## Existing Stack (Do Not Change)

The v1.0 stack is validated and working. The entries below are unchanged:

| Technology | Version | Purpose |
|------------|---------|---------|
| ratatui | 0.30 | TUI rendering |
| crossterm | 0.29 | Terminal backend / event polling |
| ratatui-textarea | 0.8 | Multi-line text editing widget |
| clap | 4.6 | CLI argument parsing |
| arboard | 3.6 | System clipboard |
| anyhow | 1.0 | Error propagation |
| thiserror | 2.0 | Typed error enums |
| tempfile | 3 | Temp files (dev-dependency) |

**Do not upgrade these** for v1.1. The lockfile is stable; there is no known bug requiring an upgrade.

---

## New Dependencies for v1.1 Features

### Standalone Commit Mode

**Feature:** `gitmedit` with no args detects git repo, writes temp file, invokes `git commit -F <tmpfile>`.

**Decision: Use `std::process::Command` (stdlib only) — no new crate needed.**

- `std::process::Command::new("git").arg("commit").arg("-F").arg(&tmpfile)` is the complete implementation.
- `git2` (libgit2 bindings) and `gix` (pure Rust gitoxide) are both overkill for a single subprocess call. `git2` links libgit2 (~2MB), which would more than double the binary size and add a C dependency. `gix` is large and still maturing for write operations.
- The temp file approach (`NamedTempFile` from the existing `tempfile` dev-dep, promoted to a runtime dep) is the standard `git commit -F` pattern used by all git hook tooling.
- **Promote `tempfile` from dev-dep to runtime dep** (no version change, already `"3"`).

```toml
# Cargo.toml change: move from [dev-dependencies] to [dependencies]
tempfile = "3"
```

Integration point: In `main.rs`, when `Cli::path` is `None` (make path optional via `Option<PathBuf>`), detect git repo root via `std::env::current_dir()`, create a `NamedTempFile`, run the TUI loop writing to it, then exec `git commit -F <path>`.

**Confidence:** HIGH — `std::process::Command` is stdlib, `tempfile` already in lockfile.

---

### Custom Hotkey Configuration

**Feature:** User can override default keybindings via a config file (e.g., `~/.config/gitmedit/config.toml`).

**Decision: `toml` + `serde` + `dirs` for config file loading; `crokey` for key combination parsing.**

#### `toml` — TOML config parsing

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| toml | 0.8 | Parse `~/.config/gitmedit/config.toml` | The toml 0.8.x series is stable and widely used. 0.9.x is newer but introduced breaking API changes; 0.8 is the safe choice for a first implementation. Provides `toml::from_str::<T>()` with serde derive. |

```toml
toml = "0.8"
```

#### `serde` — Deserialization derive

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| serde | 1.0 | Derive `Deserialize` on config structs | Already an indirect dependency (pulled by multiple existing crates). Adding it explicitly as a direct dep with `features = ["derive"]` costs nothing new at compile time. |

```toml
serde = { version = "1.0", features = ["derive"] }
```

#### `dirs` — Platform config directory

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| dirs | 6.0 | Resolve `~/.config/gitmedit/` on Linux/macOS/Windows | `dirs::config_dir()` returns the platform-correct base (`$XDG_CONFIG_HOME` or `~/.config` on Linux, `~/Library/Application Support` on macOS, `%APPDATA%` on Windows). Version 6.0 is current. |

```toml
dirs = "6.0"
```

#### `crokey` — Human-readable key combination strings in config

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| crokey | 1.4 | Parse `"ctrl-s"` strings into `crossterm::KeyEvent` | Converts human-readable key strings from TOML (e.g., `save = "ctrl-s"`) into `KeyCombination` structs that can be matched against crossterm `KeyEvent` values. Has `serde` feature for direct `HashMap<KeyCombination, Action>` deserialization. Maintained by the `broot` author — proven in production. |

```toml
crokey = { version = "1.4", features = ["serde"] }
```

**Integration point:** Load config at startup before the event loop. Merge with defaults (user config overrides, not replaces). The config struct holds a `HashMap<String, KeyCombination>` for named actions. On mismatch or missing file, silently use defaults — never fail startup due to config.

**Config file format example:**
```toml
[keys]
save = "ctrl-s"
cancel = "esc"
help = "ctrl-h"
rebase_move_up = "alt-up"
rebase_move_down = "alt-down"
```

**Confidence:** HIGH for `toml`, `serde`, `dirs`. MEDIUM for `crokey` — verified as the standard approach for crossterm keybinding config but the v1.4 version number is from a single source (GitHub Cargo.toml). Confirm with `cargo add crokey@1.4` before committing.

---

### Cross-Platform Terminal Testing

**Feature:** Tests that verify rendering output across terminal environments, used for CI and cross-platform validation.

**Decision: ratatui `TestBackend` + `insta` for snapshot assertions.**

#### `insta` — Snapshot testing

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| insta | 1.42+ | Snapshot-assert rendered terminal frames | `ratatui::backend::TestBackend` renders to an in-memory buffer. `insta::assert_snapshot!` captures that buffer as a text file and diffs on changes. This is the official pattern documented at ratatui.rs/recipes/testing/snapshots/. No PTY needed for unit-level rendering tests. Latest stable is ~1.42 (search confirmed ~1.64 in early 2026). |

```toml
[dev-dependencies]
insta = "1"
```

Use `"1"` not a specific patch — insta follows semver strictly and patch updates add reviewer features.

**What `TestBackend` covers:**
- Widget layout correctness
- Header/footer chrome rendering (nano-style bars)
- Merge toolbar content
- Rebase table row ordering after move-up/move-down
- Exec line editing state

**What it does NOT cover:**
- Terminal-specific rendering bugs (iTerm2 vs Windows Terminal)
- ANSI escape code fidelity on real terminals
- PTY behavior differences

For true cross-platform validation, manual testing on Windows Terminal (Windows) and iTerm2 (macOS) is required. `ratatui-testlib` (PTY-based) exists but is 0.1.0 and not production-ready — skip it for v1.1.

**Confidence:** HIGH — `insta` is the official ratatui testing recommendation. `TestBackend` is part of ratatui core (no new dep).

---

## Merge Comment Parsing

**Feature:** Parse MERGE_MSG comment block to extract conflict count and affected files for the status bar.

**Decision: No new dependency — pure string parsing in Rust stdlib.**

The MERGE_MSG format is a predictable git-generated comment block. Lines starting with `#` contain structured text like:
```
# Conflicts:
#	path/to/file.rs
```

Parsing this with `line.starts_with("# Conflicts:")`, `line.trim_start_matches('#').trim()` and basic string matching is sufficient. The existing `Document` model already handles comment char detection. Extend it with a `merge_metadata()` method that returns `(conflict_count, Vec<String>)`.

**No regex, no external parser.** The prior v1.0 STACK.md correctly flagged regex as over-engineering for this use case.

**Confidence:** HIGH.

---

## Nano-Style Chrome (Header / Footer Bars)

**Feature:** Persistent header showing folder name + filename; footer showing `^S Save  Esc Cancel` keybindings.

**Decision: No new dependency — ratatui `Layout`, `Block`, `Paragraph`, `Span` cover this completely.**

ratatui's `Layout::vertical([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])` splits the screen into header / content / footer. Each is a `Paragraph` with styled `Line`/`Span` content. This is standard ratatui layout — already available in the existing dep.

**Confidence:** HIGH.

---

## Rebase Line Reordering and Exec Line Editing

**Feature:** Move rebase todo rows up/down; edit the argument string of exec lines in-place.

**Decision: No new dependency — handled by `Vec` reordering in `app.rs` and a modal edit state.**

Reordering is `items.swap(selected, selected - 1)` on the existing rebase entry `Vec`. Exec line editing requires a transient edit buffer (a `String`) stored in `App` alongside the selected row index. The ratatui-textarea widget can be reused for inline editing if needed, but a plain `String` buffer with character-by-character key handling is likely sufficient and avoids modal complexity.

**Confidence:** HIGH.

---

## What NOT to Add

| Avoid | Why |
|-------|-----|
| `git2` / `gix` | Binary size (+2MB for git2, complex for gix). Single `git commit -F` call does not justify a git library. Use `std::process::Command`. |
| `regex` | Merge comment parsing is line-prefix matching. `str::starts_with` is sufficient. |
| `config` crate | The `config` crate supports layered configuration but pulls in many format-specific deps. `toml` + `serde` is more minimal and explicit for a single config file. |
| `ratatui-testlib` | PTY-based integration testing library, currently 0.1.0. API not stable. Use `TestBackend` + `insta` instead. |
| `dirs-next` | Abandoned fork. Use `dirs` (xdg-rs maintainers). |
| `toml` 0.9 | Breaking API changes from 0.8 with no clear benefit for this use case. 0.8 is the stable series. |
| `crossterm-keybind` | Low-adoption alternative to `crokey`. `crokey` is the production-proven choice (used in broot). |
| async runtime (tokio) | No async I/O needed. All git subprocess calls are synchronous. Adding tokio bloats startup. |

---

## Updated Cargo.toml

```toml
[package]
name = "gitmedit"
version = "0.2.0"
edition = "2024"

[dependencies]
ratatui = "0.30"
ratatui-textarea = { version = "0.8", features = ["crossterm"] }
crossterm = "0.29"
anyhow = "1.0"
thiserror = "2.0"
clap = { version = "4.6", features = ["derive"] }
arboard = "3.6"

# v1.1 additions
tempfile = "3"                                        # Standalone commit mode: temp file for git commit -F
toml = "0.8"                                          # Config file parsing
serde = { version = "1.0", features = ["derive"] }   # Config struct deserialization
dirs = "6.0"                                          # Platform config directory resolution
crokey = { version = "1.4", features = ["serde"] }   # Human-readable key combination strings

[dev-dependencies]
insta = "1"                                           # Snapshot testing for TUI frames

[profile.release]
lto = "thin"
codegen-units = 1
strip = true
```

**Compile-time impact estimate:** `toml` + `serde` + `dirs` + `crokey` add proc-macro compilation overhead (~15-25s incremental, one-time). Binary size increase is minimal (serde and toml are compact; dirs has no C deps; crokey is small). The 663KB release binary will likely grow to ~750-850KB — still well under 1MB.

---

## Version Compatibility

| Package | Requires | Notes |
|---------|----------|-------|
| toml 0.8 | serde 1.0 | Compatible. `toml::from_str` uses serde's Deserialize trait. |
| crokey 1.4 | crossterm 0.29+ | crokey wraps crossterm `KeyEvent`; must match the crossterm version in the lockfile. Pin both to avoid mismatch. |
| dirs 6.0 | Rust 1.63+ | No conflict with edition 2024 (requires Rust 1.85+). |
| insta 1.x | No constraints | Dev-only. Any 1.x version works. |

---

## Integration Points Summary

| Feature | New Code Location | New Dep |
|---------|------------------|---------|
| Standalone commit mode | `main.rs` — optional path arg, temp file, subprocess | `tempfile` (promoted from dev-dep) |
| Nano-style chrome | `renderer.rs` — Layout with 3 vertical constraints | None |
| Merge comment parsing | `document.rs` — `merge_metadata()` method | None |
| Rebase reordering | `app.rs` — `swap()` on rebase entries Vec | None |
| Exec line editing | `app.rs` — modal edit state with String buffer | None |
| Config loading | `config.rs` (new file) — load at startup, merge with defaults | `toml`, `serde`, `dirs`, `crokey` |
| Snapshot testing | `tests/` — `TestBackend` + `insta::assert_snapshot!` | `insta` (dev-dep) |

---

## Sources

- [crates.io — toml](https://crates.io/crates/toml) — 0.8.x stable series; 0.9.x available but breaking
- [crates.io — dirs](https://docs.rs/crate/dirs/latest) — version 6.0.0 confirmed current
- [crates.io — crokey](https://lib.rs/crates/crokey) — version 1.4 confirmed from GitHub Cargo.toml
- [github.com/Canop/crokey](https://github.com/Canop/crokey) — serde feature confirmed; used in production in broot
- [dystroy.org — Manage keybindings in a Rust terminal application](https://dystroy.org/blog/keybindings/) — crokey pattern for TOML keybinding config
- [ratatui.rs — Testing with insta snapshots](https://ratatui.rs/recipes/testing/snapshots/) — official TestBackend + insta pattern
- [crates.io — insta](https://crates.io/crates/insta) — latest ~1.64 (Feb 2026); using "1" spec
- [crates.io — tempfile](https://docs.rs/crate/tempfile/latest) — version 3.20+ current; already in dev-deps
- [docs.rs — std::process::Command](https://doc.rust-lang.org/std/process/struct.Command.html) — stdlib, no version concerns
- [crates.io — ratatui-testlib](https://crates.io/crates/ratatui-testlib) — 0.1.0, not production-ready; excluded

---

*Stack research for: gitmedit v1.1 — new feature additions only*
*Researched: 2026-04-07*
