## Why

Trying a change by hand means standing up a throwaway repo, staging files, and forcing git into a
merge, rebase, or squash before gitmedit ever opens — minutes of setup per look. The repo already
carries captured git files under `fixtures/`, but they cannot be opened directly: context detection
keys off the filename, and `rebase_fixture.txt` / `pull_rebase_fixture.txt` detect as *unknown*, so
they open as plain text with no status pane. The half-finished rename in the working tree also files
a rebase todo under `squash1/SQUASH_MSG`, where it detects as a squash message and renders the wrong
layout.

## What Changes

- Restructure `fixtures/` so every scenario is a directory holding one git file under the exact name
  git uses, making `cargo run -- ./fixtures/<scenario>/<FILE>` open the real layout with no setup.
- Fix the mislabeled fixture: the rebase todo currently at `squash1/SQUASH_MSG` moves to
  `rebase1/rebase-merge/git-rebase-todo`, the nesting git itself uses, so both context detection and
  git-dir derivation resolve the way they do in a live rebase.
- Convert the two loose `.txt` fixtures into named git files so they detect correctly instead of
  falling through to the plain-text editor.
- Fix the `ammend*` misspelling to `amend*`.
- Add the three scenarios with no fixture today: a real squash message (`SQUASH_MSG`), a tag message
  (`TAG_EDITMSG`), and an unknown file that exercises the plain-text path.
- Add `fixtures/README.md`: one copy-paste command per scenario, what each one exercises, and how to
  reset fixtures after a save edits them in place.
- Gitignore the reword store gitmedit writes beside a rebase fixture on save.
- Explicitly out of scope: no checked-in git repository. Rebase fixtures render the todo table, and
  the commit-details pane shows its unavailable state because no object database backs the hashes.
  That is the one thing these fixtures do not cover, and the README says so.

## Capabilities

### New Capabilities

None. Every behavior these fixtures exercise — filename-based context detection, git-dir derivation,
layout selection, status-block parsing, graceful degradation when git details cannot be read — is
already specified under `openspec/specs/`. This change only reshapes test data and documentation so
that specified behavior can be reached by hand.

### Modified Capabilities

None. No requirement changes; `skip_specs: true` is set in this change's `.openspec.yaml`.

## Impact

- `fixtures/` — directories renamed and restructured, three fixtures added, `README.md` added.
- `.gitignore` — one entry for the reword store written under a rebase fixture directory.
- `Cargo.toml` already excludes `fixtures` from the published crate, so the
  package contents are unaffected.
- `src/{keys,message,session,ui,rebase,status}.rs` — the unit-test modules `include_str!` the
  fixtures by path, so every moved fixture breaks the build until its path is updated. Assertions
  that hardcode fixture text must follow any content edits made during the move.
