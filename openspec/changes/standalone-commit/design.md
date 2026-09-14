## Context

Until now the editor's relationship with git is one-directional: git runs it, hands it a file, and
reads the result. This change inverts that for one flow — the editor runs git. See proposal.md — Why.
That introduces a subprocess, a temporary file, and an ordering problem between terminal teardown and
another program's output, none of which exist in the current process model.

## Goals / Non-Goals

**Goals:**

- A commit flow that is indistinguishable from `git commit` in its outcome and exit code.
- Failure paths that tell the user what git told the editor.

**Non-Goals:**

- Staging. This is an editor, not a staging tool; `git add` remains the user's job.
- Amending, signing, or any other commit option. A flag surface is a separate decision.
- Reading or writing the repository through a git library. The subprocess is the only interaction.

## Decisions

**Subprocess and a temporary file, not a git library.** The editor shells out to `git commit -F` with
a temporary file holding the message. Alternative considered: linking a git implementation, rejected
on the grounds already recorded for this project — the binary size and dependency cost are not
justified by one call, and a subprocess inherits the user's exact git configuration, hooks, and
version rather than approximating them.

**Message passed by file, not by argument.** Commit messages are multiline and contain arbitrary
text; passing one as a command-line argument invites quoting and length problems. A temporary file
sidesteps both, and it is what git's own flow uses.

**The terminal guard is dropped before the subprocess starts.** git writes to the terminal — hook
output, error messages, the commit summary — and it must do so with the terminal in its normal state.
Dropping the guard first is the ordering the whole change hangs on: teardown, then subprocess, then
exit with git's code. This is why the terminal work is a hard dependency rather than a preference.

**Exit with git's code, unmodified.** The editor does not translate git's failures into its own
codes. A caller who wrapped `git commit` and now wraps this should see no difference.

**The staged-changes check happens before the terminal is touched.** Reporting "nothing to commit"
after the user has written a message is the worst possible ordering. The check runs first, prints to
a normal terminal, and exits.

## Risks / Trade-offs

- The temporary file contains the user's message and lives on disk briefly → it is created with the
  process's own temporary file handling and removed on every path, including failure.
- Subprocess output interleaving with the editor's own teardown could garble the terminal → mitigated
  by the strict ordering above, and this is precisely what makes reliable teardown a prerequisite.
- Exiting with git's code means a caller cannot distinguish "the editor failed" from "git refused"
  → accepted deliberately; matching `git commit`'s observable behavior is the point.

## Migration Plan

Purely additive for existing users: invoking the editor with a file path behaves exactly as before.
The new behavior is reachable only through an invocation that previously was a usage error.

## Open Questions

- Whether the standalone editor should prefill git's usual commit template, including the status
  comment block, rather than starting empty. Starting empty is specified; the template variant is a
  later decision that changes neither the flow nor the failure handling.
