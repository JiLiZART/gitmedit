## MODIFIED Requirements

### Requirement: Reference is scoped to the current operation
The shortcut reference SHALL list only the actions available in the current git operation, and SHALL
state the editing restrictions that actually apply there. The restrictions it reports SHALL match the
editor's real behavior in that mode.

#### Scenario: Editing a message
- **WHEN** the reference is opened while editing a commit or merge message
- **THEN** it lists the text editing, clipboard, save, and cancel shortcuts

#### Scenario: Planning a rebase
- **WHEN** the reference is opened while editing a rebase todo
- **THEN** it lists the rebase navigation and action-cycling shortcuts
- **AND** it does not list free-text editing shortcuts, which do not apply there

#### Scenario: Restrictions are stated
- **WHEN** the current operation protects some lines from editing
- **THEN** the reference states which lines are read-only

#### Scenario: Merge mode restrictions are accurate
- **WHEN** the reference is opened in merge mode
- **THEN** it reports that comment lines are read-only
- **AND** it does not claim conflict markers are read-only, since they are editable

#### Scenario: Commit mode has no restrictions to report
- **WHEN** the reference is opened in commit mode
- **THEN** it does not claim any line is read-only, since every line is editable
