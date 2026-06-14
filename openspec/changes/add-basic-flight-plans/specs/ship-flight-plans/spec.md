## ADDED Requirements

### Requirement: Player ship flight plan model
The system SHALL represent a player ship flight plan with a stable plan id, ship id, origin state, target, departure time, calculated arrival time, calculated duration, acceleration, and status.

#### Scenario: Create object-targeted flight plan
- **WHEN** the user creates a flight plan to a known object with a valid acceleration
- **THEN** the server records a flight plan for the player ship with origin state resolved at the current authoritative simulation time, a target associated with that object, a departure time, calculated arrival time, calculated duration, acceleration, and status `active`

#### Scenario: Default departure time is current time
- **WHEN** the user creates a flight plan without an explicit departure time
- **THEN** the flight plan departure time is the current authoritative simulation time

#### Scenario: Reject invalid acceleration
- **WHEN** the user creates a flight plan with acceleration that is missing, non-finite, zero, or negative
- **THEN** the server rejects the flight plan and preserves the previous ship motion

### Requirement: Moving object intercept estimate
The system SHALL estimate object-target flight arrival by resolving the target object's predicted position at calculated arrival times before snapshotting the final target state on the flight plan.

#### Scenario: Estimate target position at arrival
- **WHEN** a flight plan targets a known moving object
- **THEN** the server iteratively estimates arrival time from origin, acceleration, and the target object's resolved state at the predicted arrival time

#### Scenario: Snapshot final target state
- **WHEN** the server registers a flight plan after estimating arrival
- **THEN** the active flight plan stores the final estimated target state and does not continuously track the target object during in-flight interpolation

#### Scenario: Deterministic iteration limit
- **WHEN** the arrival estimate does not stabilize before the configured iteration limit
- **THEN** the server uses the last deterministic estimate rather than failing the flight plan solely because the estimate did not converge

### Requirement: Flight plan position resolution
The system SHALL resolve an active flight plan as deterministic accelerated transfer motion from origin state to snapshotted target state.

#### Scenario: Resolve active transfer position
- **WHEN** the player ship state is resolved for a time between flight plan departure and arrival
- **THEN** the returned ship state position is interpolated between the flight plan origin and target using the plan acceleration profile and the returned motion mode indicates flight plan motion

#### Scenario: Resolve completed transfer
- **WHEN** the player ship state is resolved at or after the flight plan arrival time
- **THEN** the returned ship state represents arrival at the target and can transition into the target object's default fictional orbit

#### Scenario: Preserve explicit-time query behavior
- **WHEN** a query resolves player ship state at an explicit simulation time
- **THEN** the server derives the ship state from the flight plan timeline for that time without changing the authoritative simulation clock

### Requirement: Flight plan replacement and cancellation
The system SHALL allow the user to replace an active flight plan with a new one from the ship's current resolved position and to cancel the current active flight plan.

#### Scenario: Replace active flight plan
- **WHEN** the user creates a new flight plan while another flight plan is active
- **THEN** the server resolves the ship's current state at the authoritative simulation time and starts the new flight plan from that state using the newly requested acceleration

#### Scenario: Cancel active flight plan
- **WHEN** the user cancels an active flight plan
- **THEN** the server stops using that plan for future current-time ship state resolution and reports the plan status as `cancelled`

#### Scenario: Query no active flight plan
- **WHEN** the user queries flight plan status and no flight plan is active
- **THEN** the server returns a clear response indicating that there is no active flight plan
