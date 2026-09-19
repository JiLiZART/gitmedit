## Why

`gitmedit` is at version 1.0.0 but has never been published: the crate name is unclaimed on
crates.io, so the `cargo install gitmedit` that both the README and the `distribution` spec promise
fails for every user. There is also no CI, no changelog, and no repeatable way to cut a release —
every version bump would be hand-edited and hand-tagged.

## What Changes

- Adopt [knope](https://knope.tech) as the release tool, using its release-preview-pull-request
  recipe: every push to `main` refreshes an open release PR showing the next version and the
  changelog entry it would publish.
- Merging that release PR is the single release trigger: it publishes the crate to crates.io,
  tags the commit, and creates a GitHub Release whose notes come from the changelog.
- Versions are derived from conventional commits (already the project convention), not typed by
  hand — `feat` bumps minor, `fix` patch, `!`/`BREAKING CHANGE` major.
- Add a `CHANGELOG.md`, maintained by the release tool.
- Add CI (fmt, clippy, test) on pull requests and `main`; a release must not ship code that fails it.
- Fix crate packaging so what lands on crates.io is buildable and presentable: add the `readme`
  field (the registry page is otherwise blank) and exclude `.idea/`, `.claude/`, `.planning/`,
  `openspec/`, and `fixtures/` — all of which are currently packaged.
- Backfill a `v1.0.0` tag so version detection starts from a known point instead of scanning the
  whole history (the existing tags are `0.1.0` and a malformed `v1.0`).
- Document the release process and the required repository secrets.

## Capabilities

### New Capabilities
- `release-pipeline`: how a version is decided, previewed, published to the registry, and recorded
  as a tagged release with notes.

### Modified Capabilities
<!-- None. `distribution` already requires `cargo install gitmedit` to work; this change makes that
     requirement true rather than changing it. -->

## Impact

- **New files**: `knope.toml`, `CHANGELOG.md`, `.github/workflows/ci.yaml`,
  `.github/workflows/prepare_release.yaml`, `.github/workflows/release.yaml`.
- **Modified files**: `Cargo.toml` (`readme`, `exclude`), `README.md` (release/contributing notes).
- **Repository configuration** (owner action, not code): `CARGO_REGISTRY_TOKEN` is already set; a
  personal access token secret is also required, because a PR opened with the default `GITHUB_TOKEN`
  does not trigger the workflow that releases it.
- **Process**: commit messages on `main` must stay conventional — they are now the version input.
  A long-lived `release` branch is force-pushed by automation.
- **No source code changes**: `src/` is untouched.
