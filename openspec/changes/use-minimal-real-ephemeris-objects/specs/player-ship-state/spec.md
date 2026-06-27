## MODIFIED Requirements

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
