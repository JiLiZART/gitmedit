## Context

git's generated comment block is structured, but only by convention: section headings followed by
indented entries, with no markup. See proposal.md — Why. Classifying it means pattern-matching
English strings that git chooses, which is a different kind of code from the rest of this project —
it can be wrong without being broken, and it can go stale when git's wording changes.

## Goals / Non-Goals

**Goals:**

- Scannable comment blocks, where the section a line belongs to is obvious from its color.
- A classifier that degrades to today's behavior whenever it does not recognize something.

**Non-Goals:**

- Acting on the classification. Nothing is folded, hidden, jumped to, or rewritten; this is styling.
- Classifying anything outside the comment block. Content lines are the user's prose.
- Matching every git version and locale. Recognition is best-effort by construction.

## Decisions

**Classification is stateful across lines, not per line.** A file entry under "Changes to be
committed" is indistinguishable from one under "Changes not staged for commit" — the entries look
identical, and only the heading above them says which section they belong to. The classifier tracks
the current section as it walks the block, and entries inherit it. A per-line matcher cannot express
this and would color both lists the same.

**Unrecognized lines fall back to the current comment style, and that is the contract.** Every
failure mode of a string matcher — a git version with new wording, a localized install, an
unanticipated section — produces exactly what the editor renders today. This is what makes it
acceptable to match on English strings at all: being wrong costs the user nothing.

**Classification happens at render time, not at parse time.** The document model stays unchanged: a
comment is a comment. Styling is derived when drawing. This keeps the guarantee that colorization
cannot affect what is saved structural rather than something to be careful about — the saved bytes
come from a model that has never heard of sections.

**File entry kind is part of the classification, not a separate pass.** Modified, new, and deleted
entries need different colors within one section, so the entry's kind is recorded alongside its
section rather than re-derived by the renderer.

## Risks / Trade-offs

- Matching English strings breaks under a localized git → mitigated by the fallback; a non-English
  user sees today's uniform gray rather than a broken editor. Worth stating plainly in the taxonomy
  so nobody later mistakes the matcher for a parser.
- Colors are meaningful only if they are distinguishable in the user's terminal theme → keep the
  palette to widely supported colors and rely on dimming rather than fine hue distinctions for the
  de-emphasized case.
- The taxonomy was derived from four captured fixtures, which may not cover every real block →
  mitigated by the fallback, and by treating the fixtures as a growing corpus rather than a
  specification of what git can emit.

## Open Questions

- Whether the rebase instruction lines inside the status block deserve a color distinct from the
  status block itself, or should simply inherit it. Deferred: it changes one styling choice, not the
  classifier's structure.
