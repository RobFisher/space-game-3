# Ship Navigation Specification

## Purpose

Define ship navigation behavior.

## Requirements

### Requirement: Navigation module boundary
The system SHALL provide a dedicated ship navigation module that plans player ship transfers, resolves navigation phases, converts user-facing acceleration units, and calculates circular arrival orbit estimates.

#### Scenario: Plan through navigation module
- **WHEN** the server creates a player ship flight plan
- **THEN** transfer duration, arrival orbit selection, insertion state, and phase timing are produced by the navigation module rather than ad hoc query or command code

#### Scenario: Resolve navigation phase
- **WHEN** server code resolves a planned ship state for a simulation time
- **THEN** the navigation module returns the state and navigation phase for that time

### Requirement: Engine navigation profile
The system SHALL represent ship navigation engine parameters with maximum acceleration in simulation units and optional specific impulse metadata.

#### Scenario: Store maximum acceleration
- **WHEN** a ship navigation profile is configured
- **THEN** it includes a positive finite maximum acceleration expressed in kilometers per second squared

#### Scenario: Convert G acceleration input
- **WHEN** the user provides an acceleration value with a `g` suffix
- **THEN** the navigation module converts it using standard gravity and stores the equivalent kilometers per second squared value

#### Scenario: Preserve specific impulse metadata
- **WHEN** a ship navigation profile includes specific impulse
- **THEN** the navigation module validates it as positive finite seconds but does not require fuel accounting to create a plan

### Requirement: Circular arrival orbit estimates
The system SHALL resolve flight plan arrival orbit requests into circular orbit estimates around the destination object.

#### Scenario: Resolve low orbit preset
- **WHEN** a flight plan requests the `low` orbit preset
- **THEN** the navigation module resolves a circular arrival orbit using the destination object's low-orbit altitude when configured, or a deterministic fallback altitude

#### Scenario: Resolve stationary orbit preset
- **WHEN** a flight plan requests a stationary orbit and the destination object has gravitational and rotation constants
- **THEN** the navigation module resolves a circular orbit whose period matches the destination object's rotation period

#### Scenario: Reject unsupported stationary orbit
- **WHEN** a flight plan requests a stationary orbit and the required destination constants are unavailable
- **THEN** the navigation module returns a clear planning error instead of silently using another orbit

#### Scenario: Resolve custom circular orbit
- **WHEN** a flight plan requests a custom circular orbit radius or altitude
- **THEN** the navigation module resolves a circular arrival orbit with the requested size when it is positive and finite

#### Scenario: Calculate orbital estimates
- **WHEN** the destination object's gravitational parameter is known
- **THEN** the navigation module calculates circular orbital period and circular speed for the arrival orbit

### Requirement: Orbit insertion state
The system SHALL plan transfer endpoints as approximate orbit insertion states on the requested circular arrival orbit rather than as destination object center states.

#### Scenario: Choose insertion state
- **WHEN** a flight plan targets an object with an arrival orbit request
- **THEN** the navigation module stores a target state located on the resolved arrival orbit at the estimated arrival time

#### Scenario: Include insertion velocity
- **WHEN** the navigation module can calculate circular speed for the arrival orbit
- **THEN** the insertion state includes an approximate orbital velocity tangent to the circular orbit

### Requirement: Orbit entry phase
The system SHALL model a short `entering_orbit` navigation phase after transfer arrival and before exact orbiting motion.

#### Scenario: Resolve entering orbit phase
- **WHEN** the player ship state is resolved after transfer arrival but before orbit entry completes
- **THEN** the returned state blends from the transfer endpoint toward the exact circular orbit state and reports the `entering_orbit` motion mode

#### Scenario: Complete orbit entry
- **WHEN** the player ship state is resolved at or after orbit entry completion
- **THEN** the returned state uses exact circular orbiting motion around the destination object
