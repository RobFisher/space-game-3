# Ephemeris Core Specification

## Purpose

Define the game-facing ephemeris API, object registry, pure gameplay object state resolution, kernel manifest model, and backend abstraction used by the `space-game-ephemeris` crate.

## Requirements

### Requirement: Game-facing ephemeris API

The ephemeris crate SHALL expose a game-facing API that lets callers build a `SolarSystem`, query object state by string object id and `GameTime`, query relative state, compute distance, compute one-way light time, list registered objects, and inspect object metadata without using ANISE or SPICE types.

#### Scenario: Query object state

- **WHEN** a caller queries the state of a registered object at a valid `GameTime`
- **THEN** the system returns a `StateVector` containing position in kilometers, velocity in kilometers per second, frame id, epoch, and ephemeris quality

#### Scenario: Query unknown object

- **WHEN** a caller queries state or metadata for an object id that is not in the registry
- **THEN** the system returns an `UnknownObject` error that includes the requested id

#### Scenario: Compute distance and light time

- **WHEN** a caller asks for distance or light time between two registered objects at a valid `GameTime`
- **THEN** the system resolves both states through the same API and returns the Euclidean distance in kilometers or the one-way light time in seconds

### Requirement: Object registry loading

The ephemeris crate SHALL load object definitions from TOML into an object registry keyed by string `ObjectId`, including display name, object kind, ephemeris source, optional physical properties, and optional gameplay metadata.

#### Scenario: Load valid registry

- **WHEN** a registry TOML file contains valid object definitions with unique ids
- **THEN** the system loads the definitions and makes them available for state queries, object listing, and metadata lookup

#### Scenario: Reject duplicate object ids

- **WHEN** a registry TOML file defines the same object id more than once
- **THEN** the system returns an `InvalidObjectDefinition` error

#### Scenario: Preserve metadata

- **WHEN** an object definition includes physical properties or gameplay metadata
- **THEN** metadata lookup returns those values without requiring state resolution

### Requirement: Fixed offset state resolution

The ephemeris crate SHALL resolve fixed-offset objects by resolving their parent object state and adding the configured offset vector in the configured frame.

#### Scenario: Resolve fixed offset object

- **WHEN** a fixed-offset object has a registered parent and a valid offset vector
- **THEN** the returned global state position equals the parent position plus the offset and the returned velocity equals the parent velocity

#### Scenario: Detect fixed offset parent cycle

- **WHEN** fixed-offset parent references create a dependency cycle
- **THEN** the system returns a `CyclicDependency` error instead of recursing indefinitely

### Requirement: Static state resolution

The ephemeris crate SHALL resolve static-state objects as fixed inertial positions and velocities at the requested epoch.

#### Scenario: Resolve static state object

- **WHEN** a static-state object is queried at a valid `GameTime`
- **THEN** the system returns the configured position and velocity with the requested epoch

### Requirement: Circular orbit state resolution

The ephemeris crate SHALL resolve circular-orbit objects by computing parent-relative position and velocity from radius, period, inclination, right ascension of ascending node, phase at epoch, and elapsed time, then adding the parent global state.

#### Scenario: Orbit repeats after one period

- **WHEN** a circular-orbit object is queried at its epoch and again exactly one configured period later
- **THEN** the parent-relative position is equal within numeric tolerance

#### Scenario: Zero-inclination velocity is tangential

- **WHEN** a zero-inclination circular-orbit object is queried
- **THEN** its parent-relative velocity is perpendicular to its parent-relative radius within numeric tolerance

#### Scenario: Reject invalid circular orbit

- **WHEN** a circular-orbit object has a non-positive radius or non-positive period
- **THEN** registry validation or state resolution returns an `InvalidObjectDefinition` error

### Requirement: Sampled trajectory state resolution

The ephemeris crate SHALL resolve sampled-trajectory objects by linearly interpolating between bracketing samples and adding the configured centre object state.

#### Scenario: Interpolate between samples

- **WHEN** a sampled trajectory is queried at a time between two samples
- **THEN** the returned parent-relative position and velocity are linearly interpolated between those samples

#### Scenario: Return exact sample

- **WHEN** a sampled trajectory is queried exactly at a sample epoch
- **THEN** the returned parent-relative position and velocity match that sample

#### Scenario: Reject out-of-range sampled trajectory query

- **WHEN** a sampled trajectory is queried before its first sample or after its last sample
- **THEN** the system returns an `OutOfCoverage` error

### Requirement: Unsupported source behavior

The ephemeris crate SHALL define source variants and error behavior for future SPICE-backed and body-fixed objects without pretending those integrations are complete.

#### Scenario: SPICE body queried before backend implementation

- **WHEN** an object with a SPICE body source is queried before a real backend is configured or implemented
- **THEN** the system returns a clear backend or kernel-related error without exposing ANISE or SPICE types

#### Scenario: Body-fixed object queried before transforms are available

- **WHEN** an object with a body-fixed source is queried before body-fixed transforms are implemented or enabled
- **THEN** the system returns `FrameTransformUnavailable`

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
