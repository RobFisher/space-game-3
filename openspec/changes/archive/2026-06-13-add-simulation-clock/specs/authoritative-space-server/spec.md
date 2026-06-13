## ADDED Requirements

### Requirement: Simulation time command handling
The server SHALL parse and handle `time` and `advance <amount> <seconds|minutes|hours|days>` commands.

#### Scenario: Handle time command
- **WHEN** a connected client sends the command `time`
- **THEN** the server responds with the current authoritative simulation timestamp

#### Scenario: Handle advance command
- **WHEN** a connected client sends the command `advance 1 day`
- **THEN** the server advances the authoritative simulation clock by one day and responds with the updated simulation timestamp

#### Scenario: Reject invalid advance command
- **WHEN** a connected client sends an advance command with a missing amount, invalid amount, or unsupported unit
- **THEN** the server responds with a protocol error message that includes the command sequence number

## MODIFIED Requirements

### Requirement: Authoritative ephemeris boundary

The server SHALL own all runtime ephemeris queries and simulation time state for the networked TUI slice, and clients SHALL NOT need direct access to the ephemeris crate to list objects, calculate distances, or determine the authoritative simulation timestamp.

#### Scenario: Client requests object list

- **WHEN** a connected client requests known objects
- **THEN** the server queries its `SolarSystem` instance and returns object summary DTOs

#### Scenario: Client requests object distance

- **WHEN** a connected client requests the distance to a known object without an explicit timestamp
- **THEN** the server resolves the object's position at the current authoritative simulation time and returns its distance from the server-owned observer

#### Scenario: Client requests object distance at explicit time

- **WHEN** a connected client requests the distance to a known object with an explicit timestamp
- **THEN** the server resolves the object's position at the supplied timestamp without changing the authoritative simulation clock

### Requirement: Fixed observer distance queries

The server SHALL measure first-slice distances from a fixed Cartesian observer location in the ephemeris default frame.

#### Scenario: Report observer status

- **WHEN** the client requests status
- **THEN** the server response includes the current authoritative simulation time, observer label, and observer frame

#### Scenario: Calculate distance from fixed observer

- **WHEN** a target object has a resolved position in the observer frame
- **THEN** the server returns the Euclidean distance in kilometers and astronomical units from the observer position to the target position

### Requirement: Server command handling

The server SHALL parse and handle the first-slice commands `help`, `objects`, `distance <object>`, `distance <object> --at <timestamp>`, `distances`, `distances --limit <n>`, `distances --sort distance`, `distances --at <timestamp>`, `status`, `time`, and `advance <amount> <seconds|minutes|hours|days>`.

#### Scenario: Handle objects command

- **WHEN** a connected client sends the command `objects`
- **THEN** the server responds with the known object list

#### Scenario: Handle distance command

- **WHEN** a connected client sends the command `distance mars`
- **THEN** the server responds with the distance result for the object resolved from `mars`

#### Scenario: Handle distance command at explicit time

- **WHEN** a connected client sends the command `distance mars --at 2097-01-02T00:00:00Z`
- **THEN** the server responds with a distance result whose game time is `2097-01-02T00:00:00Z`

#### Scenario: Handle limited distances command

- **WHEN** a connected client sends the command `distances --limit 10`
- **THEN** the server responds with no more than 10 distance results

#### Scenario: Handle sorted distances command

- **WHEN** a connected client sends the command `distances --sort distance`
- **THEN** the server responds with distance results ordered by ascending distance

#### Scenario: Handle unknown command

- **WHEN** a connected client sends a command the server does not support
- **THEN** the server responds with a protocol error message that includes the command sequence number
