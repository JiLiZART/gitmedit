## ADDED Requirements

### Requirement: Inline rendering
The editor SHALL draw into the terminal's main screen buffer and SHALL NOT switch to the alternate
screen, so that output produced before the editor opened stays visible and remains in scrollback
after it exits.

#### Scenario: Prior output stays visible
- **WHEN** the editor opens over a terminal containing earlier output
- **THEN** that output remains on screen above the editor

#### Scenario: No alternate screen escape sequences
- **WHEN** the editor runs and exits
- **THEN** no alternate-screen enter or leave escape sequence appears anywhere in its output

#### Scenario: Session remains in scrollback
- **WHEN** the editor exits
- **THEN** the terminal retains the surrounding session output rather than restoring a screen that
  never showed it

### Requirement: Regression coverage for inline rendering
The guarantee that no alternate screen is used SHALL be covered by a test that runs as part of the
ordinary test suite, so that reintroducing the alternate screen fails the build.

#### Scenario: Ordinary test run catches a regression
- **WHEN** a change reintroduces the alternate screen and the ordinary test suite is run
- **THEN** at least one test fails

## MODIFIED Requirements

### Requirement: Terminal raw mode is always restored
The editor SHALL leave the terminal fully usable when it exits, on every path including an
unexpected panic: raw mode disabled, and any mode the editor switched on switched back off. Cleanup
SHALL NOT itself be able to fail the process — errors encountered while restoring the terminal are
suppressed rather than raised.

#### Scenario: Normal exit
- **WHEN** the editor exits after a save or a cancel
- **THEN** the terminal is no longer in raw mode and behaves as it did before the editor started

#### Scenario: Panic during editing
- **WHEN** the editor panics while the terminal is in raw mode
- **THEN** the terminal is restored before the panic message is printed
- **AND** the panic message is readable and correctly formatted

#### Scenario: Cleanup encounters an error
- **WHEN** restoring the terminal fails, for example because the output stream is already closed
- **THEN** the failure is ignored and the process exits normally rather than panicking

#### Scenario: Panic while already unwinding
- **WHEN** cleanup runs during an unwind from an earlier panic
- **THEN** cleanup does not panic, so the process is not aborted before the original panic message
  is shown
