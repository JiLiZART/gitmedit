# distribution Specification

## Purpose
Covers getting the editor onto a machine and wired into git, including working correctly under both
of the editor settings git distinguishes.

## Requirements

### Requirement: Installable with a single command
The editor SHALL be installable with a single cargo command, both from a source checkout and from
the package registry.

#### Scenario: Installing from a checkout
- **WHEN** the user runs `cargo install --path .` in a clone of the repository
- **THEN** the editor is built and installed

#### Scenario: Installing from the registry
- **WHEN** the user runs `cargo install gitmedit`
- **THEN** the published crate is built and installed

### Requirement: Available on the path
After installation the editor SHALL be invocable by name, without a path prefix, so that git can
launch it from a configuration value that names only the binary.

#### Scenario: Invoking by name
- **WHEN** the user runs `gitmedit` after installation
- **THEN** the installed binary runs

### Requirement: Configurable as git's editor
The editor SHALL work when configured as git's message editor and as git's sequence editor, and the
configuration SHALL be verifiable.

#### Scenario: Configured for messages
- **WHEN** the user sets `core.editor` to `gitmedit`
- **THEN** commit, merge, and tag messages open in this editor

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
- **THEN** message files open in the text interface and rebase todos open in the structured
  interface, with no additional configuration
