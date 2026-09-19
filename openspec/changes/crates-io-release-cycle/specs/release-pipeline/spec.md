## Purpose

Covers how a released version of the editor comes to exist: how the next version number is decided,
how a release is previewed and approved, and what a release produces — a published crate, a tag, and
release notes that match the changelog.

## ADDED Requirements

### Requirement: Version derived from commit history
The next version SHALL be derived from the conventional-commit messages merged since the last
release, following semantic versioning. No one SHALL have to edit a version number by hand.

#### Scenario: Feature commits since the last release
- **WHEN** the commits since the last release contain a `feat` commit and no breaking change
- **THEN** the next version is the current version with the minor number raised and the patch reset

#### Scenario: Only fixes since the last release
- **WHEN** the commits since the last release contain only `fix` (or other patch-level) commits
- **THEN** the next version is the current version with the patch number raised

#### Scenario: A breaking change since the last release
- **WHEN** a commit since the last release is marked as breaking
- **THEN** the next version is the current version with the major number raised

#### Scenario: Nothing releasable
- **WHEN** no commit since the last release affects the released artifact
- **THEN** no release is prepared and the automation reports success rather than failure

### Requirement: Release previewed before it ships
A pending release SHALL be visible for review before anything is published. The preview SHALL show
the next version and the release notes, and SHALL be refreshed as more changes land.

#### Scenario: Changes land on the main branch
- **WHEN** a change that affects the released artifact is merged to the main branch
- **THEN** a release pull request exists showing the next version, the updated version in the
  package manifest and lockfile, and the changelog entry that would be published

#### Scenario: More changes land before the release is approved
- **WHEN** further changes are merged while a release pull request is open
- **THEN** that pull request is updated to reflect the new next version and the combined notes,
  without opening a second one

#### Scenario: Review without publishing
- **WHEN** a release pull request is open but not merged
- **THEN** nothing is published to the registry and no release tag is created

### Requirement: Merging the release publishes it
Merging the release pull request SHALL be the only trigger that publishes a release. It SHALL
publish the crate to the package registry, create a tag for the released version, and create a
release entry whose notes match the changelog entry for that version.

#### Scenario: Release pull request merged
- **WHEN** the release pull request is merged into the main branch
- **THEN** the crate is published to the registry at the prepared version, the released commit is
  tagged with that version, and a release with the matching notes is created

#### Scenario: Ordinary pull request merged
- **WHEN** a pull request that is not a release pull request is merged
- **THEN** nothing is published and no tag is created

#### Scenario: Publishing fails
- **WHEN** the registry rejects the publish (for example the version already exists)
- **THEN** the run fails visibly and no release entry claims a version that is not on the registry

### Requirement: Released code passes its checks
The pipeline SHALL run formatting, lint, and test checks on pull requests and on the main branch,
and a release SHALL NOT be published from a commit whose checks are failing.

#### Scenario: Checks fail on a pull request
- **WHEN** formatting, clippy, or the test suite fails for a pull request
- **THEN** the pull request reports failure and is not mergeable as a passing build

#### Scenario: Checks fail on the release pull request
- **WHEN** the release pull request's checks fail
- **THEN** it is not merged and therefore nothing is published

### Requirement: Published package is installable and legible
The published package SHALL contain what is needed to build and use the binary and SHALL NOT contain
editor, planning, or specification directories. The registry listing SHALL show the project readme.

#### Scenario: Installing the published crate
- **WHEN** a user runs `cargo install gitmedit` after a release
- **THEN** the published version builds and installs the `gitmedit` binary

#### Scenario: Inspecting the package contents
- **WHEN** the package file list is inspected before publishing
- **THEN** it contains the sources, license, and readme, and contains no IDE, planning, agent, or
  specification directories

#### Scenario: Viewing the registry page
- **WHEN** a user opens the crate's page on the registry
- **THEN** the project readme is rendered there

### Requirement: Changelog kept in the repository
A changelog file SHALL be maintained in the repository, gaining one section per released version,
newest first, describing the changes in that version.

#### Scenario: A version is released
- **WHEN** a release is prepared
- **THEN** the changelog gains a section for the new version, above the previous versions, and that
  section is the source of the published release notes

### Requirement: Release credentials are scoped secrets
The pipeline SHALL authenticate to the registry and to the forge using repository secrets, and SHALL
NOT require a maintainer to hold credentials locally to cut a release.

#### Scenario: Releasing without a local login
- **WHEN** a maintainer with no registry credentials on their machine merges the release pull request
- **THEN** the release is published using the repository's stored registry token

#### Scenario: A required secret is missing
- **WHEN** a required secret is absent or expired
- **THEN** the run fails with an error naming the missing credential, and nothing is partially
  published
