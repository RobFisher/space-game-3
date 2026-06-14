# Authoritative Space Server Specification

## Purpose

Define the local authoritative server that owns ephemeris access and simulation time state for the first networked TUI slice, serves WebSocket protocol messages, and answers object, distance, and status queries from fictional demo data.

## Requirements

### Requirement: Local WebSocket server

The server SHALL expose a local WebSocket endpoint that accepts protocol messages from a client and sends protocol messages in response.

#### Scenario: Connect to local endpoint

- **WHEN** a client connects to the configured local WebSocket endpoint
- **THEN** the server accepts the connection and sends a welcome message followed by a status message

### Requirement: Authoritative ephemeris boundary

The server SHALL own all runtime ephemeris queries and simulation time state for the networked TUI slice, and clients SHALL NOT need direct access to the ephemeris crate to list objects, calculate distances, or determine the authoritative simulation timestamp.

#### Scenario: Client requests object list

- **WHEN** a connected client requests known objects
- **THEN** the server queries its `SolarSystem` instance and returns object summary DTOs

#### Scenario: Client requests object distance without explicit time

- **WHEN** a connected client requests the distance to a known object without an explicit timestamp
- **THEN** the server resolves the object's position at the current authoritative simulation time and returns its distance from the server-owned observer

#### Scenario: Client requests object distance at explicit time

- **WHEN** a connected client requests the distance to a known object with an explicit timestamp
- **THEN** the server resolves the object's position at the supplied timestamp without changing the authoritative simulation clock

### Requirement: Fictional demo registry

The server SHALL provide a demo object registry made only from ephemeris source types supported by the current ephemeris implementation.

#### Scenario: Start with demo data

- **WHEN** the server starts with default configuration
- **THEN** it initializes a solar-system model containing fictional demo objects such as the Sun, planets, Luna, and a demo station

#### Scenario: Query all demo object distances

- **WHEN** the server calculates distances for all default demo objects
- **THEN** each calculation completes without SPICE or body-fixed transform errors

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

### Requirement: Object query resolution

The server SHALL resolve object command arguments by exact object id and by case-insensitive object id or display name when unambiguous.

#### Scenario: Resolve lowercase id

- **WHEN** a client requests distance for `mars` and a demo object has id `mars`
- **THEN** the server resolves the query to that object

#### Scenario: Resolve display name with different case

- **WHEN** a client requests distance for `Mars` and a demo object has display name `Mars`
- **THEN** the server resolves the query to that object

#### Scenario: Reject ambiguous query

- **WHEN** a client object query matches more than one known object
- **THEN** the server returns an error explaining that the query is ambiguous

### Requirement: Server command completion

The server SHALL answer autocomplete requests using authoritative command metadata and runtime object data.

#### Scenario: Complete command name

- **WHEN** a connected client requests completion for input `di` with the cursor after `di`
- **THEN** the server responds with command-name candidates including `distance` and `distances`

#### Scenario: Complete object argument

- **WHEN** a connected client requests completion for input `distance ma` with the cursor after `ma`
- **THEN** the server responds with an object candidate for Mars using a replacement span that covers only `ma`

#### Scenario: Complete multi-word object display name

- **WHEN** a connected client requests completion for an object argument that matches `Demo Station`
- **THEN** the server responds with a candidate that can be inserted into the command input as a valid object query

#### Scenario: Complete option name

- **WHEN** a connected client requests completion for input `distances --s` with the cursor after `--s`
- **THEN** the server responds with the supported option candidate `--sort`

#### Scenario: Return no candidates for unsupported context

- **WHEN** a connected client requests completion for a command position the server does not support
- **THEN** the server responds with an empty completion candidate list rather than a command error

#### Scenario: Preserve completion sequence number

- **WHEN** the server answers a completion request with sequence number `22`
- **THEN** the completion response includes sequence number `22`
