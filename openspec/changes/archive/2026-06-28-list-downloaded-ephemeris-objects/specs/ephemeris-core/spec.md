## ADDED Requirements

### Requirement: Ephemeris asset object coverage metadata

The ephemeris crate SHALL parse and validate optional celestial object coverage metadata on ephemeris asset manifest entries, including object id, display name, object kind, optional NAIF id, and optional notes.

#### Scenario: Parse asset object coverage

- **WHEN** a manifest asset entry declares one or more covered celestial objects
- **THEN** the parsed manifest preserves each covered object's id, display name, object kind, optional NAIF id, and optional notes

#### Scenario: Reject invalid asset object coverage

- **WHEN** a manifest asset coverage entry has an empty object id, empty display name, unsupported object kind, or duplicate conflicting object id within the same manifest
- **THEN** manifest validation fails with a clear invalid manifest error

### Requirement: Downloaded ephemeris object listing

The project SHALL provide an explicit ephemeris asset helper command that lists celestial objects covered by valid downloaded assets for a selected manifest profile without downloading files or introspecting kernel contents.

#### Scenario: List objects from valid downloaded assets

- **WHEN** a developer runs the helper object listing command for a valid profile whose selected assets are present and valid locally
- **THEN** the command reports covered celestial objects from those valid local assets, including object id, display name, object kind, source asset id, and NAIF id when known

#### Scenario: Omit objects from missing selected assets

- **WHEN** a selected profile asset is missing locally
- **THEN** the helper object listing command omits objects declared only by that asset and reports the asset in a skipped-assets summary

#### Scenario: Omit objects from invalid selected assets

- **WHEN** a selected profile asset exists locally but fails available size or checksum verification
- **THEN** the helper object listing command omits objects declared only by that asset and reports the asset in a skipped-assets summary with the verification failure reason

#### Scenario: Report selected assets without coverage metadata

- **WHEN** a selected profile asset is valid locally but declares no celestial object coverage metadata
- **THEN** the helper object listing command reports the asset in a skipped-assets summary without listing any objects for that asset

#### Scenario: Deduplicate object listing

- **WHEN** multiple valid selected profile assets declare coverage for the same celestial object id
- **THEN** the helper object listing command reports that celestial object only once in a deterministic order

#### Scenario: Avoid network in object listing tests

- **WHEN** automated tests exercise downloaded object listing behavior
- **THEN** the tests use fixture manifests and local fixture assets rather than internet access
