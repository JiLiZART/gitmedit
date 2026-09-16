## MODIFIED Requirements

### Requirement: Configurable as git's editor
The editor SHALL work when configured as git's message editor and as git's sequence editor, and the
configuration SHALL be verifiable. The documentation SHALL recommend setting both, because inline
reword in the rebase table relies on the message editor being gitmedit too.

#### Scenario: Configured for messages
- **WHEN** the user sets `core.editor` to `gitmedit`
- **THEN** commit, merge, squash, and tag messages open in this editor

#### Scenario: Configured for sequences
- **WHEN** the user sets `sequence.editor` to `gitmedit`
- **THEN** interactive rebase todos open in this editor

#### Scenario: Verifying the configuration
- **WHEN** the user queries either setting through git
- **THEN** the configured value is reported back

### Requirement: Behavior follows the file, not the setting
The editor SHALL determine how to present a file from the file itself, so that a single installation
serves both editor settings without separate configuration or flags.

#### Scenario: Same binary under both settings
- **WHEN** the editor is configured as both `core.editor` and `sequence.editor`
- **THEN** message files open in the message layout and rebase todos open in the rebase layout, with
  no additional configuration
