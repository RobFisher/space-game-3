## MODIFIED Requirements

### Requirement: Kernel manifest model

The ephemeris crate SHALL parse and validate a profile-based ephemeris asset manifest model that describes manifest version, named profiles, profile asset membership, asset id, asset kind, filename, source, URL, local path, optional exact size, optional checksum, optional approximate size, required flag, and source or licence notes.

#### Scenario: Parse valid ephemeris asset manifest

- **WHEN** a manifest TOML file contains valid profile and asset entries
- **THEN** the system parses the manifest into structured data without performing network access

#### Scenario: Select profile assets

- **WHEN** a caller selects a manifest profile
- **THEN** the system returns the assets referenced by that profile and reports an error for any unknown asset id referenced by the profile

#### Scenario: Reject invalid ephemeris asset manifest

- **WHEN** a manifest is missing required fields, contains an unsupported asset kind, contains duplicate profile asset references, or references an unknown profile asset
- **THEN** the system returns a validation error

#### Scenario: Reject unsafe local path

- **WHEN** an asset local path is absolute or contains parent-directory traversal
- **THEN** the system returns a validation error

#### Scenario: No implicit downloads

- **WHEN** a caller builds or uses the ephemeris crate with only this capability
- **THEN** the system does not perform network access or download asset files

## ADDED Requirements

### Requirement: Ephemeris asset root resolution

The ephemeris crate SHALL resolve ephemeris asset paths relative to a repo-root default asset directory and SHALL allow that directory to be overridden by an environment variable.

#### Scenario: Resolve default asset root

- **WHEN** no ephemeris asset directory override is configured
- **THEN** the system resolves manifest asset local paths under the repository `data/ephemeris/` directory

#### Scenario: Resolve overridden asset root

- **WHEN** `SPACE_GAME_EPHEMERIS_DATA_DIR` is set
- **THEN** the system resolves manifest asset local paths under that directory instead of the repository default

#### Scenario: Report resolved asset path

- **WHEN** an asset is missing or invalid during verification
- **THEN** the system reports the asset id and resolved filesystem path in the error or command output

### Requirement: Ephemeris asset helper commands

The project SHALL provide an explicit ephemeris asset helper that can list, verify, and fetch assets from a selected manifest profile without requiring internet access for tests.

#### Scenario: List selected assets

- **WHEN** a developer runs the helper list command for a valid profile
- **THEN** the command reports the selected asset ids, descriptions, source URLs, and resolved local paths without downloading files

#### Scenario: Verify present assets

- **WHEN** all selected required assets exist and match available size and checksum metadata
- **THEN** the helper verify command exits successfully

#### Scenario: Report missing required asset

- **WHEN** a selected required asset is missing
- **THEN** the helper verify command fails with a clear message that includes the asset id, resolved local path, and suggested fetch command

#### Scenario: Report size mismatch

- **WHEN** a selected asset has `size_bytes` metadata and the local file size differs
- **THEN** the helper verify command fails with a clear size mismatch message

#### Scenario: Report checksum mismatch

- **WHEN** a selected asset has checksum metadata and the local file checksum differs
- **THEN** the helper verify command fails with a clear checksum mismatch message

#### Scenario: Fetch missing assets

- **WHEN** a developer runs the helper fetch command for a valid profile
- **THEN** the command downloads missing or invalid selected assets, verifies available size and checksum metadata, and leaves valid existing files unchanged unless a force option is used

#### Scenario: Avoid network in tests

- **WHEN** automated tests exercise manifest parsing, root resolution, verification, or fetch behavior
- **THEN** the tests use fixture manifests and local or mocked sources rather than internet access
