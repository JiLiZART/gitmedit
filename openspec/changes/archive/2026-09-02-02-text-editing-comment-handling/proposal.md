# Text Editing + Comment Handling

> Historical change. Reconstructed from `.planning/milestones/v1.0-phases/02-text-editing-comment-handling/`
> during the migration from the GSD planning system to OpenSpec. Shipped 2026-03-21 as part of v1.0.

## Why

With the git contract in place the editor can load and save a file, but not yet change one. It needs
to be an actual text editor, with the editing operations a user expects from nano.

Git message files are not plain text, though. They carry instruction comments that git strips before
committing, and merge message files carry conflict markers. Treating those lines as ordinary text
risks the one failure this tool cannot afford: handing git back a file it can no longer parse, or
silently losing information the user was relying on.

## What Changes

- Full text editing: insertion, deletion, cursor movement, Home/End, multiline messages.
- Nano-style shortcuts for line deletion, word deletion, and undo/redo.
- System clipboard integration for copy, cut, and paste, degrading silently where no clipboard exists.
- Classify every line on load as content, comment, or conflict marker, based on the comment character
  git is configured to use.
- Protect comment and conflict-marker lines from editing, and reproduce them byte for byte on save.
- Style comments and conflict markers distinctly so the user can see what is inert and what is not.
- Scroll long lines horizontally to keep the cursor visible rather than reflowing the text.

## Capabilities

### New Capabilities

- `text-editing`: the editing surface itself — keys, cursor, clipboard, and how the lines of a git
  message file are classified, protected, and written back.
- `merge-mode`: conflict-marker handling when editing a merge message.

### Modified Capabilities

- `git-context-detection`: adds reading git's configured comment character, which line classification
  depends on.

## Impact

- New modules `document.rs` (line classification and serialization) and `app.rs` (editor state).
- Dependencies introduced: ratatui-textarea for the editing surface, arboard for the clipboard.
- Requirements covered: CTX-03, CTX-04, CTX-05, CTX-06, EDIT-01 through EDIT-07, COMMIT-04,
  COMMIT-05, MERGE-01, MERGE-02, MERGE-03.
