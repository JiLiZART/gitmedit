## Context

See proposal.md — Why. Relevant current state:

- Single crate, `version = "1.0.0"` in `Cargo.toml`, `Cargo.lock` committed, never published.
- Tags are `0.1.0` and `v1.0` — neither is a version knope will recognize as the last release.
- No `.github/` directory at all: no CI, no release automation.
- `cargo package --list` currently includes `.idea/`, `.claude/`, `.planning/`, `openspec/`, and
  `fixtures/`; `Cargo.toml` has no `readme` field (it was removed in `d4e511f`).
- Conventional commits are already the project convention, so commit history is a usable version
  input without changing how anyone works.
- `CARGO_REGISTRY_TOKEN` is already stored as a repository secret.

## Goals / Non-Goals

**Goals:**
- One button to release: merge the release PR.
- Version and notes derived from history, reviewable before they ship.
- Keep the whole thing to three workflow files and one `knope.toml`.

**Non-Goals:**
- Prebuilt binaries, cross-compilation matrices, Homebrew tap, `cargo-binstall` metadata. Users
  install via `cargo install`. Add later if demand appears; the release workflow has an obvious
  insertion point (knope `assets`).
- Changesets / change files. Conventional commits alone drive versioning.
- Publishing from a maintainer's laptop as a supported path.
- Release branches for older majors.

## Decisions

### Knope's release-PR recipe over tag-push or workflow_dispatch
Chosen: knope's "preview releases with pull requests" recipe — `prepare_release.yaml` on every push
to `main` force-pushes a `release` branch and opens/updates a PR; `release.yaml` runs on that PR
merging and does the actual release.

Alternatives: (a) release on every push to `main` — no review step, a stray `feat` publishes a
version; (b) `workflow_dispatch` — a human picks the moment but the diff of what will ship is not
visible beforehand; (c) tag-push releases — puts version arithmetic back in a human's hands, which
is the thing being removed.

### Publish to crates.io *before* creating the tag/release
The `release` workflow in `knope.toml` runs `cargo publish --locked` as a `Command` step ahead of the
`Release` step. crates.io publishes are irreversible and version-unique; a failed publish should stop
the run before a tag and a GitHub Release announce a version nobody can install. The inverse failure
(published crate, missing GitHub Release) is recoverable by re-running the workflow.

### Version source: `Cargo.toml` + `Cargo.lock` as `versioned_files`
Both are listed so the committed lockfile stays consistent with the manifest in the release commit —
otherwise `cargo publish --locked` in the release workflow fails on a stale lock.

### A personal access token, not the default `GITHUB_TOKEN`
Workflow runs authenticated with the default `GITHUB_TOKEN` do not trigger other workflows, so a
release PR it opened would never run `release.yaml` on merge, and `prepare_release.yaml` needs to
push the `release` branch. The token needs contents + pull-requests write on this repository only.
This is the one piece the maintainer must add by hand; `CARGO_REGISTRY_TOKEN` is already set.

### Backfill `v1.0.0` before the first automated run
Knope reads the current version from `Cargo.toml`, but finds "changes since the last release" via
tags. With only `0.1.0` and `v1.0` present, the first changelog would sweep the entire history. An
annotated `v1.0.0` tag on the current `main` tip scopes the first automated release to commits made
after this change. Version 1.0.0 itself is published manually as part of applying this change, since
the crate name is still unclaimed and the first publish also proves the packaging fix.

### CI as a separate workflow, not a job inside the release workflows
`ci.yaml` (fmt, clippy, `cargo test`) runs on pull requests and `main`. The release PR is an ordinary
PR, so it gets the same checks; branch protection requiring them is what actually gates the release,
and that is repository configuration rather than code. PTY-based `#[ignore]`d tests stay ignored in
CI — they need a real terminal.

### Packaging hygiene via `exclude`, not `include`
`exclude` lists the directories that should not ship (`.idea`, `.claude`, `.planning`, `openspec`,
`fixtures`, `.github`). An `include` allowlist would silently drop new source directories later;
excluding known-noise directories fails safe in the other direction.

## Risks / Trade-offs

- **Force-pushed `release` branch** → it is automation-owned and disposable; nobody should branch
  from it. Documented in the README release section.
- **`prepare_release.yaml` runs on every push to `main`, including the release merge itself** → the
  recipe's `if:` guard skips commits whose message starts with the prepare-release message; the
  commit message template and the guard must stay in sync (noted in `knope.toml` and the workflow).
- **`continue-on-error` on the prepare step hides real failures** as well as "nothing to release" →
  accepted for now, it is the recipe's documented behavior; revisit with knope's `allow_empty` if a
  silent failure ever bites.
- **A publish that succeeds while the later `Release` step fails** → crate is live, GitHub Release is
  missing; re-running the workflow creates the release, and the re-run's publish failure on an
  existing version is the expected stop.
- **Conventional commits become load-bearing** → a sloppy `chore:` on a user-visible fix silently
  ships nothing. Mitigated only by convention; not worth enforcing with a commit linter yet.
- **Pinned action versions go stale** → knope's action and version are pinned for reproducibility;
  bumping them is a normal dependency chore.

## Migration Plan

1. Land packaging and config changes (`Cargo.toml`, `knope.toml`, `CHANGELOG.md`, workflows) on a
   feature branch; verify with `cargo package --list` and `cargo publish --dry-run`.
2. Add the personal access token secret; confirm `CARGO_REGISTRY_TOKEN` is present.
3. Tag `v1.0.0` on the release commit and publish 1.0.0 manually — this claims the crate name and
   proves the package builds from the registry (`cargo install gitmedit`).
4. Let the next merged change to `main` produce a release PR; merging it exercises the full pipeline
   end to end.

Rollback: delete the three workflow files. Nothing in `src/` depends on any of this, and a published
version stays installable regardless.
