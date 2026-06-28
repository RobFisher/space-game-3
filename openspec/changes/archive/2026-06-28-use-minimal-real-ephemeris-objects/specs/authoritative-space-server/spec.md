## MODIFIED Requirements

### Requirement: Fictional demo registry

The server SHALL provide a default game object registry containing the real celestial objects covered by valid local `minimal` profile ephemeris assets plus fictional station objects whose parent objects are real registered celestial objects.

#### Scenario: Start with minimal real object data

- **WHEN** the server starts with default configuration and valid local `minimal` profile assets are available
- **THEN** it initializes a solar-system model containing Sun, Mercury, Venus, Earth, Moon, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, and the default fictional station

#### Scenario: Exclude removed placeholder objects

- **WHEN** the server starts with default configuration
- **THEN** the known object list does not include placeholder-only bodies such as Ceres or the `luna` alias

#### Scenario: Fictional stations orbit real celestial objects

- **WHEN** the server resolves the default fictional station state
- **THEN** the station state is calculated from a real registered celestial parent object plus the station's configured fictional local state

#### Scenario: Query all default object distances

- **WHEN** the server calculates distances for all default objects with valid local `minimal` profile assets
- **THEN** each calculation completes without SPICE backend, missing-kernel, out-of-coverage, or body-fixed transform errors

#### Scenario: Missing minimal assets fail clearly

- **WHEN** the server needs to resolve a real default celestial object and required `minimal` profile assets are missing or invalid
- **THEN** the server reports a clear query or startup error without downloading files
