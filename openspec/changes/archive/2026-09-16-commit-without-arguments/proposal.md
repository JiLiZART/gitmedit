## Why

Writing a commit with gitmedit today means typing `git commit` and relying on `core.editor` being set
to gitmedit. Running `gitmedit` on its own is currently a usage error. Making the bare command start a
commit turns the editor into the entry point for the most common git task, and guarantees the commit
message opens in gitmedit's message layout with the status pane, whatever `core.editor` says.

## What Changes

- Running `gitmedit` with no arguments runs `git commit` in the current directory, with gitmedit set as
  git's editor for that one invocation.
- git writes the usual template, so the message opens in the normal message layout with the status
  pane. Hooks, `commit.template`, signing, and every other git setting apply as usual.
- gitmedit exits with git's exit code and leaves git's own output visible. That covers nothing to
  commit, outside a repository, hook rejections, and empty messages.
- Invoking with a file path is unchanged. More than one argument is a usage error.
- Supersedes the `standalone-commit` change, which planned an empty editor followed by
  `git commit -F`. That change is removed.

## Capabilities

### New Capabilities

- `commit-without-arguments`: starting a commit by running the editor with no arguments, and how the
  outcome is reported.

### Modified Capabilities

- `git-editor-contract`: the file argument becomes optional; its absence starts a commit.

## Impact

- Code: `src/main.rs` (optional path, spawning `git commit`), plus a new integration test.
- Processes: one `git commit` child process, with inherited terminal and `GIT_EDITOR` pointing back at
  the running gitmedit binary.
- Docs: README usage section.
- Dependencies: none added.
