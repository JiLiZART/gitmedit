## Context

Two of the four git operations this editor handles are not free-form message editing. See
proposal.md — Why. Both need a representation the ordinary text document model cannot express: a
rebase todo is a list of records, and a squash file is two regions with different permissions.

## Goals / Non-Goals

**Goals:**

- A rebase todo that cannot be corrupted by ordinary use of the editor.
- A squash file whose generated portion is structurally impossible to edit.

**Non-Goals:**

- Reordering rebase instructions, or editing the arguments of an exec instruction. Both are real
  needs and both are deferred; the interaction complexity is higher than action cycling.
- Validating that a rebase plan makes sense — that a squash follows a pick, for instance. git will
  report that, and duplicating the rule here risks disagreeing with it.

## Decisions

**A separate parse path for rebase todos, not a mode on the text document.** The text document model
classifies lines as content, comment, or conflict marker, which does not describe an instruction with
three fields. Rebase todos get their own typed line representation and their own serializer.
Alternative considered: extending the text document with a fourth line variant, rejected because
every consumer of the document model would then have to handle a variant meaningless in its context.

**Unrecognized lines are preserved verbatim rather than normalized.** The parser keeps anything it
does not recognize exactly as it found it. git's todo format has forms this editor does not model —
`break`, `label`, `reset`, `merge` — and the safe behavior for an unrecognized instruction is to hand
it back untouched. Alternative considered: rejecting files containing unknown instructions, rejected
as hostile in the middle of a rebase.

**Selection is an index into a filtered list of selectable rows, not into all rows.** Comment lines
are interleaved with instructions and must not be selectable. Keeping a separate list of selectable
positions makes "skip comments" fall out of navigation for free rather than being a special case at
each arrow press. The trade-off is that the list must be rebuilt whenever the instruction list
changes, which is why any future reordering feature has to regenerate it rather than patch it.

**Squash splitting happens before the document is parsed, not inside it.** The header is sliced off
the raw content and held separately; only the remainder becomes an editable document. This makes the
read-only guarantee structural — the header is not in the editable model at all, so no rendering or
input bug can expose it to editing. Alternative considered: marking the header lines protected inside
the document, rejected because it relies on every code path honoring the flag.

## Risks / Trade-offs

- Header detection is heuristic, and a squash file whose shape differs from the expected one falls
  back to treating everything as editable → mitigated by making the fallback safe: the user sees an
  editable file rather than an error, and the content is still round-tripped.
- Action cycling with a fixed cycle means adding an action later changes muscle memory for everyone
  → accepted; the four-action cycle covers the overwhelming majority of interactive rebases.
- Two serialization paths, one for text documents and one for rebase todos, means round-trip fidelity
  has to be proven twice → mitigated by round-trip tests on both paths.

## Open Questions

- Whether exec instructions should eventually participate in a different cycle of their own, rather
  than being inert. Deferred: it does not change the parse representation or the file format, only
  the key handling.
