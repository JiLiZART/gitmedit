## 1. Packaging hygiene

- [x] 1.1 Add `readme = "README.md"` to `Cargo.toml` and verify `cargo package --list --allow-dirty` includes `README.md`
- [x] 1.2 Add `exclude = [".idea", ".claude", ".planning", "openspec", "fixtures", ".github"]` to `Cargo.toml` and verify `cargo package --list --allow-dirty` shows none of those paths
- [x] 1.3 Run `cargo publish --dry-run --allow-dirty` and verify it completes with no errors

## 2. Release configuration

- [x] 2.1 Create `knope.toml` with `[package]` (`versioned_files = ["Cargo.toml", "Cargo.lock"]`, `changelog = "CHANGELOG.md"`) and `[github]` (`owner = "JiLiZART"`, `repo = "gitmedit"`); verify `knope --help` lists the configured workflows
- [x] 2.2 Add the `prepare-release` workflow to `knope.toml` (switch to `release` branch, `PrepareRelease`, commit `chore: prepare release $version`, force-push, `CreatePullRequest` against `main`) and verify `knope prepare-release --dry-run` prints the intended steps
- [x] 2.3 Add the `release` workflow to `knope.toml` — `Command` step `cargo publish --locked` before the `Release` step — and verify `knope release --dry-run` shows the publish running first
- [x] 2.4 Create `CHANGELOG.md` with a `1.0.0` section describing the current release, and verify `knope prepare-release --dry-run` reports it would insert the next section above it

## 3. CI and release workflows

- [ ] 3.1 Add `.github/workflows/ci.yaml` running `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` on pull requests and pushes to `main`; verify the run passes on the change's own pull request
- [ ] 3.2 Add `.github/workflows/prepare_release.yaml` (push to `main`, skip-guard on the prepare-release commit message, full-history checkout with the PAT, git identity, `knope-dev/action`, `knope prepare-release --verbose`) and verify the file parses by pushing the branch and seeing the workflow registered
- [ ] 3.3 Add `.github/workflows/release.yaml` (on `pull_request` `closed`, guarded on `merged == true` and head branch `release`, `knope release --verbose` with `GITHUB_TOKEN` and `CARGO_REGISTRY_TOKEN`) and verify it appears in the repository's workflow list
- [x] 3.4 Confirm the skip-guard string in `prepare_release.yaml` matches the commit message template in `knope.toml` exactly

## 4. Repository configuration (maintainer actions)

- [ ] 4.1 Create a fine-grained personal access token limited to this repository with contents and pull-requests write, store it as a repository secret, and verify the secret is listed in repository settings
- [ ] 4.2 Verify `CARGO_REGISTRY_TOKEN` is present and valid by confirming `cargo publish --dry-run` authenticates (or re-issue the token if it fails)
- [ ] 4.3 Enable branch protection on `main` requiring the CI checks, and verify a failing check blocks merge on a test pull request

## 5. First release

- [ ] 5.1 Merge this change to `main`, tag the resulting commit `v1.0.0`, push the tag, and verify `git tag --list "v1.0.0"` resolves on the remote
- [ ] 5.2 Publish 1.0.0 manually (`cargo publish --locked`) and verify the crate page exists on crates.io with the README rendered
- [ ] 5.3 Verify `cargo install gitmedit` in a clean environment installs a working `gitmedit` binary (`gitmedit --version`)

## 6. Pipeline verification

- [ ] 6.1 Merge any conventional-commit change to `main` and verify a release pull request appears with the expected next version and changelog entry
- [ ] 6.2 Merge the release pull request and verify the workflow publishes the new version to crates.io, tags it, and creates a GitHub Release whose notes match the changelog section
- [ ] 6.3 Push a `chore:`-only change to `main` and verify no release pull request is created and the workflow still reports success

## 7. Documentation

- [x] 7.1 Document the release process in `README.md` (conventional commits drive the version, merge the release PR to ship, the `release` branch is automation-owned and force-pushed) and verify the section renders correctly on GitHub
- [x] 7.2 Note the two required secrets and their scopes in the same section, and verify no token values appear anywhere in the repository
