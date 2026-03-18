# Stack Research

**Domain:** Rust TUI git editor (invoked as `git core.editor`)
**Researched:** 2026-03-18
**Confidence:** HIGH (all versions verified against crates.io; architecture patterns verified against official ratatui docs)

---

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| ratatui | 0.30.0 | TUI rendering framework | The de-facto standard for Rust TUIs since the tui-rs fork in 2023. 11.9M+ downloads. 0.30.0 reorganized into a modular workspace — only pay for what you use. Immediate-mode rendering with sub-millisecond frame times satisfies the startup speed requirement. |
| crossterm | 0.29.0 | Terminal backend / raw mode / event polling | Cross-platform (Linux, macOS, Windows 7+). Ships as ratatui's default backend. Handles raw mode, alternate screen, and keyboard event normalization. The only backend worth defaulting to for cross-platform support. |
| ratatui-textarea | 0.8.0 | Multi-line text editor widget | Maintained by the official ratatui org. Provides exactly what gitmedit needs: multi-line editing, undo/redo, cursor line highlight, styled rendering, crossterm event passthrough. Eliminates reimplementing text buffer logic. Depends on ratatui's modular workspace crates (`ratatui-core ^0.1`, `ratatui-widgets ^0.3`). |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| anyhow | 1.0.102 | Ergonomic error handling with context chains | Use everywhere errors propagate up to `main`. Wraps underlying errors with `.context("reading COMMIT_EDITMSG")`. Simple `?`-based propagation. |
| thiserror | 2.0.18 | Typed domain error enums | Use to define `AppError` variants for git-specific failure modes (file not found, corrupt rebase todo, empty message on save). `thiserror` derives `Display`/`Error` for you; `anyhow` wraps them for propagation. |
| clap | 4.6.0 | CLI argument parsing | Git invokes the editor as `gitmedit <filepath>`. clap parses that positional argument and any future flags (`--version`, `--help`). Use the `derive` feature for zero-boilerplate argument structs. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| cargo (workspace) | Build system | Project uses Cargo 2024 edition workspace. Set `resolver = "3"` in workspace root Cargo.toml (default for edition = "2024"). Shared `[workspace.dependencies]` for version pinning across future crates. |
| `profile.release` tuning | Binary size + startup speed | Set `lto = "thin"`, `codegen-units = 1`, `strip = true` in release profile. Reduces binary size significantly; startup time is dominated by process spawn overhead, not Rust init. |
| cargo-dist or plain `cargo install` | Distribution | For v1: `cargo install gitmedit` is sufficient. Users set `git config --global core.editor gitmedit`. No complex packaging needed. |

---

## Installation (Cargo.toml)

```toml
[package]
name = "gitmedit"
version = "0.1.0"
edition = "2024"

[dependencies]
ratatui = "0.30"
ratatui-textarea = { version = "0.8", features = ["crossterm"] }
crossterm = "0.29"
anyhow = "1.0"
thiserror = "2.0"
clap = { version = "4.6", features = ["derive"] }

[profile.release]
lto = "thin"
codegen-units = 1
strip = true
```

Note: ratatui 0.30 ships as a re-export workspace. `ratatui = "0.30"` pulls the full facade. `ratatui-textarea = "0.8"` pins to `ratatui-core ^0.1` internally — both resolve against the same lockfile without conflict.

---

## Alternatives Considered

| Category | Recommended | Alternative | When to Use Alternative |
|----------|-------------|-------------|-------------------------|
| TUI framework | ratatui 0.30 | tui-rs (fdehau/tui-rs) | Never — archived, unmaintained since 2023. ratatui is its continuation. |
| TUI framework | ratatui 0.30 | cursive | If you prefer a retained-mode widget model. cursive has a steeper widget abstraction; ratatui's immediate-mode is simpler for a focused tool. |
| TUI framework | ratatui 0.30 | iced (TUI mode) | iced targets GUI-class applications with Elm architecture. Overhead is not justified for a single-screen editor. |
| Terminal backend | crossterm | termion | termion is Unix-only. crossterm is the only choice if Windows support is a stretch goal. |
| Text widget | ratatui-textarea 0.8 | tui-textarea 0.7 | tui-textarea 0.7 is stable and still maintained. Use it if you hit a ratatui-textarea bug — API is near-identical. Last published Oct 2024; ratatui-textarea publishes more frequently. |
| Text widget | ratatui-textarea 0.8 | hand-rolled buffer | Only if you need behavior ratatui-textarea can't provide (e.g., per-character color spans in the buffer). For gitmedit's needs, the widget is sufficient and saves ~2 weeks of work. |
| Error handling | anyhow + thiserror | failure, eyre | `failure` is deprecated. `eyre` is a valid swap for `anyhow` but adds no value for a CLI tool. Stick with anyhow's ubiquity. |
| Arg parsing | clap 4.6 | pico-args, lexopt | Valid for ultra-minimal binaries. gitmedit only has one positional arg; clap is overkill but pays off for `--help`, `--version`, and future flags without manual parsing. |

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| tui-rs (original) | Archived. Author handed off to ratatui team in 2023. No security patches. | ratatui 0.30 |
| tokio / async runtime | gitmedit is invoked synchronously by git. Adding an async runtime adds startup overhead and complexity for a tool that processes a single file. crossterm's synchronous event polling (`read()`) is sufficient. | crossterm sync event loop |
| termwiz | Wez Furlong's terminal library, integrated into WezTerm. No community adoption outside that ecosystem. Documentation sparse. | crossterm |
| serde + config files | PROJECT.md explicitly scopes out configuration file complexity. Adding serde pulls in proc-macro compile time for zero user value. | Hard-coded defaults |
| Regex for comment parsing | Git comment lines are trivially `line.starts_with('#')`. Pulling in the `regex` crate for this is over-engineering. | `str::starts_with` |

---

## Stack Patterns by Variant

**If Windows support is required (stretch goal):**
- crossterm already handles it — no change needed
- Test with Windows 10+ terminal; Windows 7 support is theoretical
- Key event normalization: on Windows, crossterm may emit both `KeyEventKind::Press` and `KeyEventKind::Release`; filter to `Press` only

**If rebase todo editing becomes complex (future milestone):**
- Consider a separate `Mode` enum (`CommitMsg | MergeMsg | RebaseTodo | Squash`) in the app state
- Each mode can configure ratatui-textarea differently (read-only comment lines, syntax highlighting via custom line spans)
- Still no new crates needed — ratatui's `Span` and `Line` APIs handle inline styling

**If binary size becomes critical (sub-1MB target):**
- Replace clap with pico-args (saves ~300KB in release build)
- Use `panic = "abort"` in release profile
- ratatui's modular workspace already helps by not pulling unused widget code

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| ratatui 0.30 | crossterm 0.29 | ratatui 0.30 ships `ratatui-crossterm ^0.1` which depends on crossterm 0.29. Pin both to avoid mismatch. |
| ratatui-textarea 0.8 | ratatui 0.30 | ratatui-textarea 0.8 depends on `ratatui-core ^0.1` (the modular crate from ratatui 0.30 workspace). Compatible. |
| clap 4.6 | Rust 2024 edition | clap 4.x requires Rust 1.74+. Cargo edition 2024 implies 1.85+. No conflict. |
| thiserror 2.0 | anyhow 1.0 | No coupling — both implement `std::error::Error`. thiserror types can be wrapped by anyhow transparently. |

---

## Git Editor Contract (Critical Context)

gitmedit is invoked by git as: `gitmedit /path/to/COMMIT_EDITMSG`

Git's expectations:
- **Exit 0**: File has been edited and saved. Git reads the file contents as the commit message.
- **Exit non-zero**: Editor signaled failure or cancellation. Git aborts the commit.
- **Empty message detection**: Git itself detects if the written message is empty/all-comments and aborts — the editor does not need to handle this (but may warn).

This means `std::process::exit(0)` on save and `std::process::exit(1)` on Esc-cancel is the complete integration contract. No git library dependency needed for v1.

---

## Sources

- [crates.io — ratatui](https://crates.io/crates/ratatui) — version 0.30.0 confirmed (published 2025-12-26)
- [crates.io — crossterm](https://crates.io/crates/crossterm) — version 0.29.0 confirmed (published 2025-04-05)
- [crates.io — ratatui-textarea](https://crates.io/crates/ratatui-textarea) — version 0.8.0 confirmed (published 2026-02-21)
- [crates.io — tui-textarea](https://crates.io/crates/tui-textarea) — version 0.7.0 (published 2024-10-22); ratatui-textarea is more current
- [crates.io — anyhow](https://crates.io/crates/anyhow) — version 1.0.102 confirmed
- [crates.io — thiserror](https://crates.io/crates/thiserror) — version 2.0.18 confirmed
- [crates.io — clap](https://crates.io/crates/clap) — version 4.6.0 confirmed (published 2026-03-12)
- [ratatui.rs — Application Patterns](https://ratatui.rs/concepts/application-patterns/) — Elm/Flux/Component patterns; event handler recipe
- [ratatui.rs — Terminal and Event Handler recipe](https://ratatui.rs/recipes/apps/terminal-and-event-handler/) — Tui struct pattern with crossterm EventStream
- [github.com/ratatui/ratatui-textarea](https://github.com/ratatui/ratatui-textarea) — Official ratatui org fork of tui-textarea; confirmed maintained
- [The Cargo Book — Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html) — resolver = "3" default for edition 2024

---

*Stack research for: Rust TUI git editor (gitmedit)*
*Researched: 2026-03-18*
