## Purpose

Gives the user feedback about the commit message they are writing, so that conventions about subject
length and body separation are visible while typing rather than discovered in review.

## ADDED Requirements

### Requirement: Live subject length
While editing a commit message, the editor SHALL display the character count of the subject line and
SHALL update it as the user types.

#### Scenario: Typing in the subject
- **WHEN** the user types on the first line of the message
- **THEN** the displayed count reflects the current length of that line

#### Scenario: Message with no content yet
- **WHEN** the message contains no editable content
- **THEN** the count reads zero rather than showing an error

### Requirement: Subject length is colored by convention
The subject character count SHALL be colored to indicate whether the subject is within convention:
comfortable at 50 characters or fewer, cautionary from 51 to 72, and over budget beyond 72.

#### Scenario: Short subject
- **WHEN** the subject is 50 characters or fewer
- **THEN** the count is shown in the comfortable color

#### Scenario: Borderline subject
- **WHEN** the subject is between 51 and 72 characters
- **THEN** the count is shown in the cautionary color

#### Scenario: Over-long subject
- **WHEN** the subject exceeds 72 characters
- **THEN** the count is shown in the over-budget color

### Requirement: Missing blank separator is surfaced
When a message has a body, the editor SHALL indicate whether the conventional blank line separating
the subject from the body is present.

#### Scenario: Body follows immediately
- **WHEN** the second line of the message has visible content
- **THEN** the editor indicates that the blank separator is missing

#### Scenario: Separator present
- **WHEN** the second line of the message is blank
- **THEN** no missing-separator indication is shown

#### Scenario: Single-line message
- **WHEN** the message has fewer than two lines
- **THEN** no missing-separator indication is shown, since a subject alone needs no separator
