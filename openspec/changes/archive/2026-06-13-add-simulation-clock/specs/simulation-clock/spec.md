## ADDED Requirements

### Requirement: Server-owned simulation clock
The server SHALL maintain an authoritative simulation clock initialized from the configured default game time.

#### Scenario: Initialize simulation clock
- **WHEN** the server starts with default configuration
- **THEN** the simulation clock starts at the configured default game time

#### Scenario: Compute running simulation time
- **WHEN** wall-clock time elapses after the simulation clock is initialized
- **THEN** querying the clock returns a simulation timestamp advanced by the elapsed duration at the configured simulation rate

### Requirement: Deterministic clock calculation
The simulation clock SHALL calculate current simulation time from explicit clock inputs so tests can verify elapsed-time behavior without sleeping.

#### Scenario: Query with controlled wall time
- **WHEN** a test queries the simulation clock with a controlled wall-clock instant one second after its anchor instant
- **THEN** the returned simulation timestamp is exactly one simulation second after the anchor simulation timestamp at rate `1.0`

#### Scenario: Advance with controlled wall time
- **WHEN** a test advances the simulation clock by one day using a controlled wall-clock instant
- **THEN** the new simulation timestamp is exactly one day after the clock's current timestamp at that instant

### Requirement: Simulation time query
The server SHALL allow clients to query the current authoritative simulation time.

#### Scenario: Client requests simulation time
- **WHEN** a client sends a simulation time request
- **THEN** the server responds with the current simulation timestamp, whether the clock is running, and the simulation rate

### Requirement: Manual simulation time advancement
The server SHALL allow clients to advance the simulation clock by seconds, minutes, hours, or days.

#### Scenario: Advance by seconds
- **WHEN** a client advances simulation time by `30 seconds`
- **THEN** the server moves the authoritative simulation timestamp thirty seconds forward from the current timestamp

#### Scenario: Advance by larger units
- **WHEN** a client advances simulation time by `2 hours`
- **THEN** the server moves the authoritative simulation timestamp two hours forward from the current timestamp

#### Scenario: Reject unsupported time unit
- **WHEN** a client requests a time advance using an unsupported unit
- **THEN** the server responds with a protocol error and does not change the authoritative simulation timestamp
