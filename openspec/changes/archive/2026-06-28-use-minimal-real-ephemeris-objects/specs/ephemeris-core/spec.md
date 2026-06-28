## MODIFIED Requirements

### Requirement: Unsupported source behavior

The ephemeris crate SHALL resolve SPICE-backed body objects from configured valid local ephemeris assets, while continuing to define clear error behavior for SPICE-backed objects whose required assets are missing, invalid, unconfigured, or out of coverage and for body-fixed objects whose transforms are not implemented.

#### Scenario: SPICE body queried with configured local assets

- **WHEN** an object with a SPICE body source is queried for an epoch covered by configured valid local assets
- **THEN** the system returns a `StateVector` in the requested game-facing frame with `RealKernel` quality without exposing ANISE or SPICE types

#### Scenario: SPICE body queried without required local assets

- **WHEN** an object with a SPICE body source is queried and the required local assets are missing, invalid, or unconfigured
- **THEN** the system returns a clear kernel-related error that identifies the missing or invalid asset without downloading files

#### Scenario: SPICE body queried outside coverage

- **WHEN** an object with a SPICE body source is queried outside the configured asset coverage
- **THEN** the system returns an `OutOfCoverage` error for that object and epoch

#### Scenario: Body-fixed object queried before transforms are available

- **WHEN** an object with a body-fixed source is queried before body-fixed transforms are implemented or enabled
- **THEN** the system returns `FrameTransformUnavailable`
