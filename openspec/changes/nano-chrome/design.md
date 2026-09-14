## Context

Every mode currently renders into a two-region layout: content, and a one-line status bar. See
proposal.md — Why. Adding a header turns that into three regions, and the content renderers compute
scroll offsets and cursor positions from the area they are handed. Any mode that receives the wrong
region, or the full frame, will place the cursor in the wrong row and scroll against the wrong
height — visibly, and only at the bottom of a long message.

## Goals / Non-Goals

**Goals:**

- One layout decision, made once, that every mode inherits.
- Chrome that cannot silently desynchronize from the content area's arithmetic.

**Non-Goals:**

- Configurable chrome, colors, or key labels. The bars show what the mode offers, with no settings.
- Merge conflict file information in the toolbar. That belongs to the merge toolbar work.

## Decisions

**The three-region split is computed once, at the top level, and sub-renderers receive the middle
region.** No mode computes its own layout or looks at the full frame. This is the decision the
previous planning system recorded as a pre-v1.1 constraint, for a concrete reason: the content
renderers derive both the visible height and the cursor's screen row from the area passed in, so a
mode that receives the frame instead of the content region is off by exactly the header height, in
both. Alternative considered: letting each mode split the frame itself, rejected because it makes
that mistake available in five places instead of impossible in one.

**Chrome is drawn from state the modes already hold.** The header needs the file path, the command
bar needs the detected operation, and both are already available. Nothing new is threaded through,
so there is no new coupling between chrome and content.

**The command bar and the shortcut reference derive from one source.** Both list keys per operation,
and they are the two places a user learns what the editor does. Keeping them independent is how the
reference came to claim conflict markers are read-only after they became editable. One definition of
"what this mode offers", rendered two ways.

## Risks / Trade-offs

- Two lines of vertical space are lost to chrome, which matters most in rebase mode where every line
  is a commit → accepted; it is the same trade nano makes, and the alternative is an editor that
  never says what file it has open.
- Cursor and scroll arithmetic changes in every mode at once → mitigated by making the layout split
  a single top-level decision, and by testing cursor placement at the bottom edge of a long message
  where an off-by-header-height error becomes visible.

## Open Questions

- Whether the header should show a path relative to the repository root rather than just the folder
  and file name, for worktrees and nested repositories. Deferred: it changes the header's text, not
  the layout or the approach.
