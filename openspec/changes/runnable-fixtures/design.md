## Context

See proposal.md — Why. The constraints that shape the layout come from two functions in
`src/context.rs`:

- `detect_context` matches the **exact final path segment** (`COMMIT_EDITMSG`, `MERGE_MSG`,
  `SQUASH_MSG`, `TAG_EDITMSG`, `git-rebase-todo`). Directories are ignored, extensions are not
  tolerated — `rebase_fixture.txt` can only ever detect as unknown.
- `git_dir` derives the git directory *positionally*: the file's parent for message contexts, the
  parent's parent for a rebase todo. So a todo has to sit one level deeper than a message file for
  its git dir to land on the same directory.

Everything the right-hand pane shows in message mode is parsed out of the file's own comment block
(`src/status.rs`) — no git process involved. Only rebase mode shells out, via `git --git-dir <dir>
show <hash>` in `src/details.rs`, and via the reword store in `src/reword.rs`. That split is what
makes fixtures-without-a-repo viable at all.

## Goals / Non-Goals

**Goals:**
- One directory per scenario, openable by path alone, no environment setup, no git state.
- Directory shapes that mirror what git actually produces, so the code paths taken are the real ones.
- Cover every branch of `detect_context`, including unknown.
- Keep the working tree clean after a fixture session.

**Non-Goals:**
- A checked-in git repository, real commit hashes, or a populated commit-details pane in rebase mode.
- A runner script, make target, or any new binary flag — the path is the interface.
- Automated tests over the fixtures. They are for hands-on inspection; unit tests already cover
  parsing with inline data.

## Decisions

### Directory per scenario, named for the git file it holds

`fixtures/<scenario>/<EXACT_GIT_FILENAME>`, keeping the numbered-scenario convention already started
in the working tree.

*Why:* `detect_context` gives no other lever. Putting the file in its own directory also makes
`git_dir` resolve to that directory alone, so each scenario is self-contained — anything gitmedit
writes beside the fixture lands in that scenario's folder and nowhere else.

*Alternative rejected:* a `--context` override flag to open arbitrary filenames. That adds a
production code path that exists only for development, and it would bypass the very detection logic
the fixtures are meant to exercise.

### The rebase todo nests under `rebase-merge/`

`fixtures/rebase1/rebase-merge/git-rebase-todo`, not `fixtures/rebase1/git-rebase-todo`.

*Why:* `git_dir` takes the grandparent for a rebase todo. Flat nesting would resolve the git dir to
`fixtures/` itself, putting the reword store and any details lookup outside the scenario folder. The
nested form matches git's own layout and keeps the scenario contained.

### Final fixture map

| Scenario path | Detected context | What it exercises |
|---|---|---|
| `amend1/COMMIT_EDITMSG` | Commit | staged-file list, branch ahead-of-remote line |
| `amend2/COMMIT_EDITMSG` | Commit | larger diff, diverged-branch line |
| `merge1/MERGE_MSG` | Merge | conflict section, merge boilerplate stripping |
| `merge2/MERGE_MSG` | Merge | conflicts resolved, deleted-file badges |
| `rebase1/rebase-merge/git-rebase-todo` | Rebase | todo table, pick/reword/squash keys (was `squash1/SQUASH_MSG`) |
| `rebase2/COMMIT_EDITMSG` | Commit | rebase-in-progress status, submodule sections, warnings (was `rebase_fixture.txt`) |
| `rebase3/MERGE_MSG` | Merge | minimal file: conflicts only, no boilerplate (was `pull_rebase_fixture.txt`) |
| `squash1/SQUASH_MSG` | Squash | squash layout — **new**, no fixture exists today |
| `tag1/TAG_EDITMSG` | Tag | tag layout — **new** |
| `plain1/notes.txt` | Unknown | full-width plain editor, no right pane — **new** |

The three new fixtures are hand-written to match real git output for those operations; the rest are
moves of existing captured content, byte-for-byte.

### `rebase2` keeps `COMMIT_EDITMSG`, not a rebase name

Its content is git's commit-message template as written during an interactive rebase — commit
context, with rebase state showing in the status block. Naming it `git-rebase-todo` to match the
directory would misroute it to the rebase table and render nonsense. Directory names describe the
situation; **file names are dictated by git**.

## Risks / Trade-offs

- **Saving a fixture rewrites it in place.** Inspecting a fixture and pressing save leaves a dirty
  working tree. → README documents `git checkout fixtures/` as the reset; the diff makes any
  accidental save obvious in `git status`.
- **Rebase reword writes `fixtures/rebase1/gitmedit/reword/<hash>`** on save. → `.gitignore` entry
  for `fixtures/*/gitmedit/`.
- **Commit details are dead in rebase mode.** `git --git-dir fixtures/rebase1 show <hash>` fails
  with *not a git repository*; `details::fetch` returns `None` and the pane renders unavailable.
  → Accepted and documented. Verified that the failure is clean: stderr is suppressed, nothing
  panics, the todo table stays usable.
- **Fixture content references a former employer's package names.** Already present in the
  checked-in fixtures and unchanged by the moves; not this change's problem to solve, but worth
  noting before the crate's fixtures ever ship. `Cargo.toml` already excludes `fixtures` from the
  published package.
- **`git mv` history.** The working tree already holds a half-finished rename staged as renames.
  Completing it with `git mv` keeps `--follow` history intact for the moved files.

## Migration Plan

Not applicable — test data and documentation only. Rollback is `git checkout fixtures/ .gitignore`.
