## 1. Fixtures

- [ ] 1.1 Commit the pending fixture changes, including the new merge fixture, before starting
- [ ] 1.2 Confirm the fixture set covers commit, amend, merge, rebase, and pull-rebase blocks

## 2. Section classifier

- [ ] 2.1 Define the section taxonomy: instructions, branch info, merge notice, rebase status, rebase instruction, conflicts, staged changes, unstaged changes, submodule info
- [ ] 2.2 Walk the comment block tracking the current section
- [ ] 2.3 Attribute indented entries to the heading above them
- [ ] 2.4 Record the entry kind for file lines: modified, new, deleted
- [ ] 2.5 Fall back to unrecognized for anything that does not match

## 3. Styling

- [ ] 3.1 Map each section to a style
- [ ] 3.2 Dim the instruction preamble
- [ ] 3.3 Share one neutral style between branch and submodule information
- [ ] 3.4 Use a warning style for conflicts
- [ ] 3.5 Style file entries by kind
- [ ] 3.6 Give rebase status its own distinct style
- [ ] 3.7 Render unrecognized comments in the existing default style

## 4. Verification

- [ ] 4.1 Classifier tests over each fixture, asserting the section of every comment line
- [ ] 4.2 Tests that an unrecognized block classifies entirely as unrecognized without erroring
- [ ] 4.3 Round-trip test confirming a classified message saves byte for byte
- [ ] 4.4 Test that an edited comment line saves exactly as typed
- [ ] 4.5 Confirm the full existing suite still passes
