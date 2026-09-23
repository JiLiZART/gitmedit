## 1. Rename and reshape the existing fixtures

- [x] 1.1 `git mv fixtures/ammend1 fixtures/amend1` and `fixtures/ammend2 fixtures/amend2`; verify `ls fixtures/amend1 fixtures/amend2` each show `COMMIT_EDITMSG` and `git status` records renames, not delete+add
- [x] 1.2 Move the mislabeled rebase todo: create `fixtures/rebase1/rebase-merge/` and `git mv fixtures/squash1/SQUASH_MSG fixtures/rebase1/rebase-merge/git-rebase-todo`; verify the file's first line is still `pick 54763e6 ...` and `fixtures/squash1/` is left empty for task 2.1
- [x] 1.3 `git mv fixtures/rebase_fixture.txt fixtures/rebase2/COMMIT_EDITMSG` (creating the directory); verify the content is unchanged with `git diff --cached -M --stat` showing a pure rename
- [x] 1.4 `git mv fixtures/pull_rebase_fixture.txt fixtures/rebase3/MERGE_MSG` (creating the directory); verify no loose `.txt` files remain under `fixtures/` via `ls fixtures/*.txt` returning nothing
- [x] 1.5 Confirm `merge1/MERGE_MSG` and `merge2/MERGE_MSG` already match the target map in design.md and need no move; verify by listing `fixtures/merge1 fixtures/merge2`

## 2. Add the three missing scenarios

- [ ] 2.1 Write `fixtures/squash1/SQUASH_MSG` with real `git rebase --squash` output — the combined subjects of two or more commits, blank-line separated, followed by the `# This is a combination of N commits.` comment block and a staged-file status section; verify `cargo run -- ./fixtures/squash1/SQUASH_MSG` opens the message layout with a populated status pane
- [ ] 2.2 Write `fixtures/tag1/TAG_EDITMSG` with git's annotated-tag template — a subject line plus the `# Write a message for tag:` comment block; verify `cargo run -- ./fixtures/tag1/TAG_EDITMSG` opens the message layout and the comment block renders in the right pane, not the editor
- [ ] 2.3 Write `fixtures/plain1/notes.txt` with a few lines of ordinary prose including at least one `#`-prefixed line; verify `cargo run -- ./fixtures/plain1/notes.txt` opens a full-width plain editor with no right pane and the `#` line stays in the editor

## 3. Keep the working tree clean

- [ ] 3.1 Add `fixtures/*/gitmedit/` to `.gitignore`; verify by opening `fixtures/rebase1/rebase-merge/git-rebase-todo`, marking a line `reword`, saving, and confirming `git status --short` reports nothing under `fixtures/rebase1/gitmedit/`
- [ ] 3.2 Reset any fixture content edited during verification with `git checkout fixtures/`; verify `git status --short fixtures/` shows only the intended renames and additions

## 4. Document the fixtures

- [ ] 4.1 Write `fixtures/README.md` with one table row per scenario: path, detected context, the exact `cargo run -- ./fixtures/...` command, and what it exercises — matching the fixture map in design.md; verify every listed path exists by piping the paths through `ls`
- [ ] 4.2 Document in that README that saving edits the fixture in place and `git checkout fixtures/` resets it; verify the reset command works from a dirty fixture
- [ ] 4.3 Document in that README that rebase fixtures show an empty commit-details pane because no object database backs the hashes, and that this is the one behavior fixtures do not cover

## 5. Verify every scenario end to end

- [ ] 5.1 Open each of the ten fixtures in turn and confirm the layout matches the Detected context column of the fixture map — message layout for commit/merge/squash/tag, rebase table for the todo, plain full-width editor for `notes.txt`
- [ ] 5.2 Confirm the rebase todo opens the table and the details pane degrades cleanly rather than hanging or panicking; verify with `git --git-dir fixtures/rebase1 show HEAD` returning *not a git repository* and the TUI still responding to navigation keys
- [ ] 5.3 Run `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`; verify all pass — no source changed, so this only confirms the change introduced no regression
- [ ] 5.4 Run `cargo package --list` and verify no `fixtures/` entry appears, confirming the existing `exclude` still keeps them out of the published crate
