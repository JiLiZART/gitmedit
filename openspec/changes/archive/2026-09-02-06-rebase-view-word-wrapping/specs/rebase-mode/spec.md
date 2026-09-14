## ADDED Requirements

### Requirement: Long subjects wrap in the table
Commit subjects too wide for the subject column SHALL be wrapped across multiple lines rather than
truncated, so the full subject of every instruction is readable.

#### Scenario: Subject longer than the column
- **WHEN** a commit subject does not fit the available subject width
- **THEN** it is displayed across as many lines as it needs, with no text omitted

#### Scenario: Breaking at a word boundary
- **WHEN** a wrap point falls inside a word and a space exists earlier within the available width
- **THEN** the break is taken at that space rather than mid-word

#### Scenario: Word longer than the column
- **WHEN** a single word is itself wider than the available width
- **THEN** it is broken at the width, since no word boundary is available

#### Scenario: Subject that fits
- **WHEN** a subject fits within the available width
- **THEN** it is displayed on a single line, unchanged

### Requirement: Rows take the height their content needs
Table rows SHALL be sized to fit their wrapped subject, and scrolling SHALL account for rows of
differing heights so that selection and display stay aligned.

#### Scenario: Row height follows the wrap
- **WHEN** a subject wraps across several lines
- **THEN** its row occupies that many terminal lines

#### Scenario: Scrolling with mixed row heights
- **WHEN** the table contains both wrapped and unwrapped rows and the user moves the selection
- **THEN** the selected row remains visible and the rows stay correctly aligned with their content

#### Scenario: Very narrow terminal
- **WHEN** the terminal leaves no usable width for the subject column
- **THEN** the table still renders without error
