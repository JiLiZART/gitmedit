## ADDED Requirements

### Requirement: Conflict markers are editable
Conflict-marker lines in a merge message SHALL be editable, so the user can remove them as part of
resolving the conflict. Comment lines SHALL remain protected and SHALL be written back unchanged.

#### Scenario: Markers can be edited
- **WHEN** the user moves the cursor onto a conflict-marker line in a merge message
- **THEN** the cursor enters the line and it can be edited

#### Scenario: Markers can be removed
- **WHEN** the user deletes the conflict-marker lines while resolving and saves
- **THEN** the file written back no longer contains them

#### Scenario: Comments still survive a save
- **WHEN** the user edits a merge message and saves
- **THEN** every comment line is written back unchanged and in its original position

## REMOVED Requirements

### Requirement: Conflict markers are protected
**Reason**: Resolving a conflict means deleting the markers. Protecting them forced the user out to
another editor to finish the job the editor was opened for.
**Migration**: Replaced by "Conflict markers are editable" in the same capability. Comment lines in
merge messages keep their existing protection, so no file structure git depends on becomes editable.
