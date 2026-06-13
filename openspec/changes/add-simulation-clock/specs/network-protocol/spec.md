## ADDED Requirements

### Requirement: Simulation time protocol messages
The protocol SHALL represent client requests for simulation time, client requests to advance simulation time, and server responses containing simulation time state.

#### Scenario: Serialize simulation time request
- **WHEN** a client simulation time request with sequence number 4 is serialized to JSON and deserialized again
- **THEN** the resulting message preserves the original sequence number

#### Scenario: Serialize simulation time response
- **WHEN** a server simulation time response containing sequence number, current time, running state, and rate is serialized to JSON and deserialized again
- **THEN** the resulting message preserves all simulation time fields

#### Scenario: Serialize simulation time advance
- **WHEN** a client simulation time advance request containing amount `2` and unit `hours` is serialized to JSON and deserialized again
- **THEN** the resulting message preserves the sequence number, amount, and unit

### Requirement: Explicit distance query time
The protocol SHALL allow distance requests and distances requests to optionally include an explicit simulation timestamp.

#### Scenario: Serialize distance request without explicit time
- **WHEN** a distance request omits an explicit timestamp
- **THEN** the resulting protocol message represents that the server should use its current simulation time

#### Scenario: Serialize distance request with explicit time
- **WHEN** a distance request includes `2097-01-02T00:00:00Z` as its explicit timestamp
- **THEN** serialization and deserialization preserve that timestamp

## MODIFIED Requirements

### Requirement: Request correlation

Client-initiated requests that expect command-specific responses SHALL include a sequence number, and corresponding server responses SHALL include the same sequence number.

#### Scenario: Correlate object list response

- **WHEN** the server responds to an object list request with sequence number 7
- **THEN** the object list response includes sequence number 7

#### Scenario: Correlate simulation time response

- **WHEN** the server responds to a simulation time request with sequence number 6
- **THEN** the simulation time response includes sequence number 6

#### Scenario: Correlate error response

- **WHEN** the server rejects a command associated with sequence number 8
- **THEN** the error response includes sequence number 8

### Requirement: Status messages

The protocol SHALL represent server status messages containing game time, observer label, observer frame, object count, and connection-relevant server information.

#### Scenario: Receive unsolicited status

- **WHEN** the server sends a status update that is not a direct response to a command
- **THEN** the message can omit a request sequence number while preserving status fields

#### Scenario: Receive status with authoritative simulation time

- **WHEN** the server sends a status update
- **THEN** the status message includes the current authoritative simulation time
