## Context

Line protection was implemented as a property of a line's kind: a comment was protected everywhere,
a conflict marker was protected everywhere. See proposal.md — Why. That is now wrong in two
directions at once, and the correction has to be made without weakening the one guarantee that
matters, which is that a squash or merge file comes back to git in the shape git produced.

## Goals / Non-Goals

**Goals:**

- Protection becomes a property of the operation, not of the line kind.
- The squash and merge paths keep their existing round-trip behavior exactly.

**Non-Goals:**

- Removing the notion of protected lines. Squash still needs it, and so do merge comments.
- Reworking the status bar into a command bar. This change only empties it; filling it is the next
  change's work.

## Decisions

**An editing mode carried on the document, rather than deleting the protected line kinds.** The line
classification stays as it is; what changes is a mode value, decided from the detected operation and
passed in when the file is parsed, that decides which kinds enter the editing surface. Alternative
considered: removing the comment and conflict-marker line kinds entirely and treating everything as
content, rejected outright — squash and merge round-tripping is built on those kinds, and removing
them globally would corrupt both.

**Mode is decided once, at open, and never changes.** The mapping from operation to mode is a single
match in one place. Making the mode fixed for the lifetime of the document means the index of
editable lines is built once and cannot drift from what is displayed.

**Plain mode short-circuits serialization.** In plain mode every line came from the editing surface,
so reassembly is just the surface's lines joined back together, with no re-injection of protected
content. The re-injection path is left untouched for the modes that still need it. Alternative
considered: running plain mode through the same re-injection loop with an empty protected set,
rejected because the loop's correctness depends on the editable index staying in step with the line
list, and a path that never needs that coupling should not be subject to it.

**Merge promotes conflict markers to editable but leaves comments protected.** This is the only mode
with a split, and it is deliberate: the markers are the user's work, the comment block is git's.

## Risks / Trade-offs

- Commit messages lose byte-for-byte comment preservation, since comments now round-trip through the
  editing surface → accepted, and in fact the point: git strips those lines before committing, so
  nothing downstream depends on their bytes.
- Three modes mean three serialization behaviors to keep correct → mitigated by keeping the existing
  merge and squash tests unchanged, so any regression in them fails loudly.
- The help overlay's text about read-only conflict markers becomes false with this change → not
  mitigated here; carried as a known inconsistency into the chrome work.

## Migration Plan

No data or file migration. The change is observable to users immediately on upgrade: comment lines in
commit messages become editable, and the subject counter disappears.
