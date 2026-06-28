## MODIFIED Requirements

### Requirement: Player ship flight plan model
The system SHALL represent a player ship flight plan with a stable plan id, ship id, origin state, object target, departure time, calculated transfer arrival time, orbit entry completion time, calculated duration, acceleration, arrival orbit request, resolved arrival orbit estimate, navigation phase timing, and status.

#### Scenario: Create object-targeted flight plan
- **WHEN** the user creates a flight plan to a known object with a valid acceleration
- **THEN** the server records a flight plan for the player ship with origin state resolved at the current authoritative simulation time, a target associated with that object, a departure time, calculated transfer arrival time, orbit entry completion time, calculated duration, acceleration, arrival orbit metadata, and status `active`

#### Scenario: Default departure time is current time
- **WHEN** the user creates a flight plan without an explicit departure time
- **THEN** the flight plan departure time is the current authoritative simulation time

#### Scenario: Reject invalid acceleration
- **WHEN** the user creates a flight plan with acceleration that is missing, non-finite, zero, or negative
- **THEN** the server rejects the flight plan and preserves the previous ship motion

#### Scenario: Record G acceleration display value
- **WHEN** the user creates a flight plan with acceleration expressed in G
- **THEN** the flight plan stores normalized acceleration in kilometers per second squared and exposes the equivalent G value for display

### Requirement: Moving object intercept estimate
The system SHALL estimate object-target flight arrival by resolving the target object's predicted position at calculated arrival times, resolving the requested arrival orbit at that time, and snapshotting the final orbital insertion state on the flight plan.

#### Scenario: Estimate target position at arrival
- **WHEN** a flight plan targets a known moving object
- **THEN** the server iteratively estimates arrival time from origin, acceleration, requested arrival orbit, and the target object's resolved state at the predicted arrival time

#### Scenario: Snapshot final target state
- **WHEN** the server registers a flight plan after estimating arrival
- **THEN** the active flight plan stores the final estimated orbital insertion state and does not continuously track the target object during in-flight interpolation

#### Scenario: Deterministic iteration limit
- **WHEN** the arrival estimate does not stabilize before the configured iteration limit
- **THEN** the server uses the last deterministic estimate rather than failing the flight plan solely because the estimate did not converge

### Requirement: Flight plan position resolution
The system SHALL resolve an active flight plan as deterministic phase-based navigation from origin state to snapshotted orbital insertion state, through orbit entry, and then into circular orbiting motion.

#### Scenario: Resolve active transfer position
- **WHEN** the player ship state is resolved for a time between flight plan departure and transfer arrival
- **THEN** the returned ship state position is interpolated between the flight plan origin and insertion target using the plan acceleration profile and the returned motion mode indicates flight plan motion

#### Scenario: Resolve orbit entry position
- **WHEN** the player ship state is resolved after transfer arrival but before orbit entry completion
- **THEN** the returned ship state is derived from the plan orbit-entry phase and the returned motion mode indicates entering orbit motion

#### Scenario: Resolve completed transfer
- **WHEN** the player ship state is resolved at or after the flight plan orbit entry completion time
- **THEN** the returned ship state represents the configured circular arrival orbit around the target object

#### Scenario: Preserve explicit-time query behavior
- **WHEN** a query resolves player ship state at an explicit simulation time
- **THEN** the server derives the ship state from the flight plan timeline for that time without changing the authoritative simulation clock

## ADDED Requirements

### Requirement: Configurable arrival orbits
The system SHALL allow player ship flight plans to request default, low, stationary, or custom circular arrival orbits around the target object.

#### Scenario: Default arrival orbit
- **WHEN** the user creates a flight plan without an orbit option
- **THEN** the flight plan uses the configured default arrival orbit

#### Scenario: Low arrival orbit
- **WHEN** the user creates a flight plan requesting a low orbit
- **THEN** the flight plan records a low circular arrival orbit request and its resolved orbit estimate

#### Scenario: Stationary arrival orbit
- **WHEN** the user creates a flight plan requesting a stationary orbit around a supported target
- **THEN** the flight plan records a stationary circular arrival orbit whose period matches the target body's rotation period

#### Scenario: Custom arrival orbit
- **WHEN** the user creates a flight plan with a valid custom orbit altitude or radius
- **THEN** the flight plan records the custom circular arrival orbit request and resolved orbit estimate
