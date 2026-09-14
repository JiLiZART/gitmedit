# Installation + Distribution

> Historical change. Reconstructed from `.planning/milestones/v1.0-phases/05-installation-distribution/`
> during the migration from the GSD planning system to OpenSpec. Shipped 2026-03-27 as part of v1.0.

## Why

An editor that is not installed as the git editor is not used. The last step of v1.0 is making the
tool reachable: installable with one command, on the path afterwards, and configurable as git's
editor for both the operations git distinguishes.

git uses two separate settings. `core.editor` is consulted for messages — commits, merges, tags — and
`sequence.editor` for the interactive rebase todo. A tool that handles both must work when set as
either, which it can only do by deciding its behavior from the file it is given rather than from
which setting invoked it.

## What Changes

- Publishable crate metadata: name, version, description, license, repository, keywords, categories.
- Installable from a source checkout with `cargo install --path .` and from the registry with
  `cargo install gitmedit`.
- Documented setup for `core.editor` and `sequence.editor`, with commands to verify each.
- Release profile tuned for a small, fast binary.

## Capabilities

### New Capabilities

- `distribution`: how the editor is installed and wired into git as the editor for both message and
  sequence operations.

### Modified Capabilities

None.

## Impact

- Crate metadata and release profile in `Cargo.toml`; installation and configuration sections in the
  README.
- Requirements covered: INSTALL-01, INSTALL-02, INSTALL-03, INSTALL-04, INSTALL-05.

**Correction made during migration:** the original requirement text specified
`cargo install --path crates/gitmedit`, a workspace path this project never adopted. The spec records
the single-crate form that actually works, `cargo install --path .`.
