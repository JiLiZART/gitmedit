# Colorize Comment Sections

> Carried over from GSD Phase 13 (`.planning/ROADMAP.md`) during the migration to OpenSpec. Not started.

## Why

The comment block git prepends to a commit message is dense and uniform. It mixes boilerplate the
user has read a thousand times with the information they actually opened the editor to check: which
branch, which files are staged, what conflicted, how far through a rebase they are. All of it renders
in the same dim gray, so scanning it means reading it.

The sections are trivially distinguishable — git emits them with stable, recognizable headings — and
coloring them by meaning turns a wall of text into something readable at a glance. This became worth
doing once comment lines became ordinary editable content: they are no longer inert scenery the user
is expected to ignore.

## What Changes

- Classify each comment line into a semantic section: instructions, branch and author information,
  merge notice, rebase status, conflicts, staged changes, unstaged changes, submodule information,
  and the rebase instruction list.
- Render each section in a color that matches its meaning: instructions de-emphasized, informational
  lines neutral, conflicts as a warning, staged additions and modifications positive, deletions
  negative, rebase status distinct from all of them.
- Keep the change purely visual: classification affects rendering only and never what is saved.

## Capabilities

### New Capabilities

- `comment-colorization`: how the comment block git generates is classified and colored.

### Modified Capabilities

None. Comment lines remain ordinary editable content; only their styling changes.

## Impact

- A classifier over comment lines, and per-section styling in the renderer.
- No change to parsing, serialization, or the editing model.
- Requirements covered: UI-10, UI-11, UI-12.
- Depends on: `08-plain-editor-default`, already archived — comment lines had to become editable
  content before it made sense to treat them as something the user reads and edits.

**Note:** the section taxonomy was derived from real git output captured in `fixtures/`. Those
fixtures currently have uncommitted modifications in the working tree, including a new
`merge_fixture2.txt`; they appear to be the raw material for this work and should be committed
before implementation starts, so the classifier is developed against a fixed set.
