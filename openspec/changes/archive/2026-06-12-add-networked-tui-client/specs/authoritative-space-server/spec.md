## ADDED Requirements

### Requirement: Local WebSocket server

The server SHALL expose a local WebSocket endpoint that accepts protocol messages from a client and sends protocol messages in response.

#### Scenario: Connect to local endpoint

- **WHEN** a client connects to the configured local WebSocket endpoint
- **THEN** the server accepts the connection and sends a welcome message followed by a status message

### Requirement: Authoritative ephemeris boundary

The server SHALL own all runtime ephemeris queries for the networked TUI slice, and clients SHALL NOT need direct access to the ephemeris crate to list objects or calculate distances.

#### Scenario: Client requests object list

- **WHEN** a connected client requests known objects
- **THEN** the server queries its `SolarSystem` instance and returns object summary DTOs

#### Scenario: Client requests object distance

- **WHEN** a connected client requests the distance to a known object
- **THEN** the server resolves the object's position at the current game time and returns its distance from the server-owned observer

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
- **THEN** the server response includes the observer label and observer frame

#### Scenario: Calculate distance from fixed observer

- **WHEN** a target object has a resolved position in the observer frame
- **THEN** the server returns the Euclidean distance in kilometers and astronomical units from the observer position to the target position

### Requirement: Server command handling

The server SHALL parse and handle the first-slice commands `help`, `objects`, `distance <object>`, `distances`, `distances --limit <n>`, `distances --sort distance`, and `status`.

#### Scenario: Handle objects command

- **WHEN** a connected client sends the command `objects`
- **THEN** the server responds with the known object list

#### Scenario: Handle distance command

- **WHEN** a connected client sends the command `distance mars`
- **THEN** the server responds with the distance result for the object resolved from `mars`

#### Scenario: Handle limited distances command

- **WHEN** a connected client sends the command `distances --limit 10`
- **THEN** the server responds with no more than 10 distance results

#### Scenario: Handle sorted distances command

- **WHEN** a connected client sends the command `distances --sort distance`
- **THEN** the server responds with distance results ordered by ascending distance

#### Scenario: Handle unknown command

- **WHEN** a connected client sends a command the server does not support
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
