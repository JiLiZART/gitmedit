## Context

Rebase mode holds three pieces of state that must agree: the instruction list, the list of positions
that are selectable, and the current selection. Until now only the third ever changed. Reordering
changes the first, which invalidates the second and can strand the third. See proposal.md — Why.

## Goals / Non-Goals

**Goals:**

- Reordering that cannot desynchronize navigation from the instruction list.
- Reordering that is provably lossless on files containing comments.

**Non-Goals:**

- Adding or deleting instructions. Dropping a commit is already expressible as the drop action.
- Validating the resulting plan. git reports an invalid sequence; this editor does not duplicate that
  judgment.
- Editing anything about a non-exec instruction beyond its action.

## Decisions

**Reorder by exchanging instructions, not adjacent list positions.** The instruction list contains
interleaved comment lines. A naive swap of neighbouring positions moves an instruction into a comment
line's slot, which reorders the comments along with the commits and changes what the file means. The
move instead finds the previous or next *instruction* position and exchanges the two, leaving every
other line where it was. This is the case worth writing a test for first: `pick A`, a comment,
`pick B`, with B moved up.

**Regenerate the selectable index from scratch after every move.** Not patched incrementally. This
was recorded as a constraint by the previous planning system and it is the right one: an incremental
update has to be correct for every combination of move direction, comment placement, and list
boundary, while a regeneration has to be correct once. Reordering is a keypress-frequency operation
on a list of tens of items, so the cost is irrelevant.

**Selection is re-derived from the moved instruction's new position.** After regenerating the index,
the selection is set to wherever the moved instruction now sits. Alternative considered: adjusting
the selection index by one in the direction of travel, rejected because it is only correct when
exactly one selectable position was crossed, which is not guaranteed with comments in play.

**Inline exec editing is a distinct input state, not a mode.** While an exec command is being edited,
rebase mode's navigation keys must not act — Alt+Up should type nothing and move nothing. Modelling
it as a state on the rebase view, with its own key handling and its own exit conditions, keeps that
containment explicit rather than scattering "unless editing" conditions through the key handler.

**Conflicted files are parsed for display only.** The comment block remains a comment: protected,
preserved byte for byte, never rewritten from the parsed representation. Parsing is a read.

## Risks / Trade-offs

- Reordering interacts with the wrapped-subject row heights: rows change position, and their heights
  travel with them → mitigated by regenerating row heights from the instruction list on each render
  rather than caching them by position.
- A second input state in rebase mode means the key handler has two shapes, and a missed case sends
  navigation keys into the text being edited → mitigated by making the editing state own its key
  handling entirely rather than filtering the existing path.
- Conflict-block parsing depends on git's comment wording, which is localized → the parse must fail
  softly: an unrecognized block displays nothing, which is the same as no block at all.

## Open Questions

- Whether Alt+Up and Alt+Down survive terminal key handling everywhere, particularly on Windows
  consoles. If not, a fallback binding is needed. This is left to cross-platform verification rather
  than guessed at now.
