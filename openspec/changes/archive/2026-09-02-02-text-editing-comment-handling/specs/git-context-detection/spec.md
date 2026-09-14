## ADDED Requirements

### Requirement: Configured comment character
The editor SHALL read git's configured comment character at startup and use it to recognize comment
lines, falling back to `#` whenever the configuration cannot be resolved.

#### Scenario: Custom comment character configured
- **WHEN** git is configured with a comment character other than `#`
- **THEN** lines beginning with that character are recognized as comments

#### Scenario: No configuration set
- **WHEN** git reports no configured comment character
- **THEN** `#` is used

#### Scenario: Automatic mode configured
- **WHEN** git is configured to choose the comment character automatically
- **THEN** `#` is used, since automatic selection is not supported

#### Scenario: git unavailable
- **WHEN** git cannot be executed to read the configuration
- **THEN** `#` is used and the editor starts normally
