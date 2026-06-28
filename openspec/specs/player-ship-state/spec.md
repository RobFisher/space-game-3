# Player Ship State Specification

## Purpose

Define the server-owned single-player ship identity, naming, motion mode, and spatial state resolution used as the gameplay subject for the current networked slice.

## Requirements

### Requirement: Authoritative player ship state

The system SHALL maintain one server-owned player ship with a stable ship id, editable display name, current motion mode, and resolvable spatial state.

#### Scenario: Default player ship exists

- **WHEN** the server starts with default configuration
- **THEN** it initializes one player ship with a stable id, non-empty display name, and orbiting motion mode

#### Scenario: Ship state resolves at simulation time

- **WHEN** server code requests the player ship state at a valid `GameTime`
- **THEN** the system returns a state containing position, velocity, frame, epoch, quality, ship id, ship name, and motion mode

### Requirement: Orbiting ship motion

The system SHALL resolve an orbiting player ship as a fictional circular orbit around a registered parent object by adding the parent object's resolved spatial state to the ship's parent-relative orbital state, including when the parent object is backed by real local ephemeris assets.

#### Scenario: Default ship orbits real Earth

- **WHEN** the default player ship state is resolved at the default simulation time with valid local `minimal` profile assets
- **THEN** the ship state is in a fictional orbit whose parent object is the real registered Earth object

#### Scenario: Orbiting ship moves with simulation time

- **WHEN** the player ship state is resolved at two different simulation times
- **THEN** the returned epochs match the requested times and the orbiting ship position changes according to its configured orbit

#### Scenario: Orbiting ship uses parent state

- **WHEN** the player ship orbits a registered parent object
- **THEN** the ship's global state is calculated from the parent object's state at the same simulation time plus the ship's parent-relative orbital state

### Requirement: Runtime ship naming

The system SHALL allow the single player ship display name to be changed at runtime without changing its stable ship id or motion state.

#### Scenario: Rename player ship

- **WHEN** the user names the ship `Wayfarer`
- **THEN** subsequent ship status and server status responses use `Wayfarer` as the ship display name

#### Scenario: Reject empty ship name

- **WHEN** the user attempts to set the ship name to an empty or whitespace-only value
- **THEN** the system rejects the change with a command error and preserves the previous ship display name

#### Scenario: Ship name is not persisted

- **WHEN** the server restarts after a runtime ship rename
- **THEN** the default player ship display name is restored

### Requirement: Flight plan-derived ship state

The system SHALL resolve the player ship state from active flight plan motion when a flight plan is active, while preserving orbiting motion for ships without an active flight plan.

#### Scenario: Active flight plan determines ship state

- **WHEN** the player ship has an active flight plan and server code requests ship state at a time during the transfer
- **THEN** the returned state contains position, velocity, frame, epoch, quality, ship id, ship name, and flight plan motion mode derived from that flight plan

#### Scenario: Orbiting motion remains default

- **WHEN** the player ship has no active flight plan
- **THEN** ship state resolution continues to use the existing orbiting motion behavior

#### Scenario: Completed flight hands off to orbiting motion

- **WHEN** the active flight plan has reached its arrival time
- **THEN** subsequent current-time ship state can be represented as orbiting motion around the flight plan target object

### Requirement: Ship state scope

The system SHALL treat the player ship as mutable game state rather than a public ephemeris registry object for this change.

#### Scenario: Ship omitted from object list

- **WHEN** a client requests the known object list
- **THEN** the response contains registered demo objects but does not include the player ship as a normal object

#### Scenario: Object lookup remains registry-only

- **WHEN** a client uses an object argument for `where <object>` or `distance <object>`
- **THEN** the server resolves that argument against registered objects, not against the player ship
