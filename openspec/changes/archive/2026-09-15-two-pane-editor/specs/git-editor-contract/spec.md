## ADDED Requirements

### Requirement: Terminal is always restored
When the editor exits by any path, including an unexpected panic, it SHALL disable mouse capture,
leave the alternate screen, show the cursor, and disable raw mode, so the user returns to the shell
exactly as they left it. The cleanup SHALL NOT panic: errors during restoration are suppressed so
that restoration can still run while a panic is unwinding.

#### Scenario: Normal exit
- **WHEN** the editor exits after a save or a cancel
- **THEN** the terminal is out of raw mode, off the alternate screen, and not capturing the mouse
- **AND** the output shown before the editor started is visible again

#### Scenario: Panic during editing
- **WHEN** the editor panics while the terminal is set up
- **THEN** the terminal is fully restored before the panic message is printed
- **AND** the panic message is readable in the normal screen

#### Scenario: Restoration step fails
- **WHEN** one restoration step returns an error
- **THEN** the remaining steps still run and the process does not abort

## MODIFIED Requirements

### Requirement: Atomic write-back
On save, the editor SHALL write the content back to the same path it was given, atomically, so that
an interrupted write cannot leave the file truncated or partially written.

#### Scenario: Saving edited content
- **WHEN** the user saves after editing
- **THEN** the file at the original path contains exactly the content the editor assembled

#### Scenario: No temporary files left behind
- **WHEN** a save completes successfully
- **THEN** no temporary or intermediate file remains beside the target file

#### Scenario: Line endings preserved
- **WHEN** content is written back
- **THEN** unix line endings are emitted and no additional trailing content is introduced

### Requirement: Fast startup
The editor SHALL draw its first frame in well under 100 milliseconds on typical hardware, so that it
does not feel heavier than the nano or vim it replaces. Work that needs git beyond reading the comment
character, such as loading rebase commit details, SHALL NOT delay the first frame.

#### Scenario: Cold invocation
- **WHEN** the editor binary is invoked
- **THEN** the time from process start to a drawn first frame stays under 100 milliseconds

#### Scenario: Rebase todo with many commits
- **WHEN** a rebase todo listing many commits is opened
- **THEN** the first frame is drawn without waiting for any commit details to load

## REMOVED Requirements

### Requirement: Terminal raw mode is always restored
**Reason**: The editor now also owns the alternate screen and mouse capture, which must be restored
too.
**Migration**: See "Terminal is always restored".
