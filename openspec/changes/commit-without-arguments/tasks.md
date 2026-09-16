Conventions: write the failing test first, and end each group with a conventional commit that has no
AI attribution lines.

## 1. Invocation (`src/main.rs`)

- [x] 1.1 Make the clap argument `path: Option<PathBuf>`. With a path, keep the current flow unchanged. Verify: `cargo test` still passes, and `target/debug/gitmedit a b` exits non-zero with a usage error.
- [x] 1.2 Add `fn shell_quote(path: &Path) -> String`, which wraps the path in single quotes and turns each embedded `'` into `'\''`. Verify unit tests for `/usr/bin/gitmedit` → `'/usr/bin/gitmedit'`, a path with spaces, and `/a'b` → `'/a'\''b'`.

## 2. Commit mode (`src/main.rs`, `tests/commit_without_arguments.rs`)

- [x] 2.1 Implement the no-argument path:
  - Spawn `git commit` in the current directory with `GIT_EDITOR` set to `shell_quote(current_exe())` and inherited stdio.
  - Create no `TerminalGuard`.
  - Exit with git's exit code, or 1 if git was killed by a signal.
  - If spawning fails, print `gitmedit: could not run git: <error>` to stderr and exit 127.

  Verify `cargo build` succeeds.
- [x] 2.2 Add integration tests that run `env!("CARGO_BIN_EXE_gitmedit")` with no arguments. Each returns early if git can't be run.
  - In a temp repo with nothing staged: the exit code equals what `git commit` itself returns, and the output contains `nothing to commit`.
  - In a temp directory that isn't a repository, with `GIT_CEILING_DIRECTORIES` set to its parent: the exit code is non-zero and stderr contains `not a git repository`.

  Verify `cargo test --test commit_without_arguments` passes.

## 3. Docs and verification

- [x] 3.1 Add the no-argument commit to `README.md` Usage, noting that it overrides `core.editor` for that run only and that git's usual output and exit codes apply. Verify by reviewing the README.
- [x] 3.2 Run a PTY end-to-end check in a temp repo that has a staged file and a local `core.editor` set to `vi`. Verify:
  - `gitmedit` with no arguments opens the message layout, showing `Staged (1)`.
  - Typing a subject and pressing Ctrl+S exits 0, and `git log -1 --format=%s` shows that subject.
  - A second run cancelled with Esc creates no commit and exits non-zero.
  - After both runs, the terminal is restored (mouse off, alternate screen left, cursor shown, echo on).
- [x] 3.3 Run `cargo test`, `cargo clippy --all-targets` and `openspec validate commit-without-arguments --strict`; verify all pass with no new warnings.
