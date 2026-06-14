## MODIFIED Requirements

### Requirement: Authoritative ephemeris boundary

The server SHALL own all runtime ephemeris queries, simulation time state, and player ship state for the networked TUI slice, and clients SHALL NOT need direct access to the ephemeris crate to list objects, calculate distances, determine the authoritative simulation timestamp, or resolve the player ship state.

#### Scenario: Client requests object list

- **WHEN** a connected client requests known objects
- **THEN** the server queries its `SolarSystem` instance and returns object summary DTOs

#### Scenario: Client requests object distance without explicit time

- **WHEN** a connected client requests the distance to a known object without an explicit timestamp
- **THEN** the server resolves the object's position and the player ship position at the current authoritative simulation time and returns the distance from the player ship

#### Scenario: Client requests object distance at explicit time

- **WHEN** a connected client requests the distance to a known object with an explicit timestamp
- **THEN** the server resolves the object's position and the player ship position at the supplied timestamp without changing the authoritative simulation clock

### Requirement: Location summary

The server SHALL provide an authoritative location summary for the player ship or a named object by resolving the subject spatial state and comparing it with known object states at the effective simulation time.

#### Scenario: Report ship location summary

- **WHEN** a connected client sends the command `where`
- **THEN** the server responds with the player ship label, subject type, frame, simulation time, nearest known object, distance in kilometers, distance in astronomical units, and spatial quality

#### Scenario: Report object location summary

- **WHEN** a connected client sends the command `where mars`
- **THEN** the server responds with a location summary for the object resolved from `mars`

#### Scenario: Report object location summary at explicit time

- **WHEN** a connected client sends the command `where mars --at 2097-01-02T00:00:00Z`
- **THEN** the server responds with a location summary whose game time is `2097-01-02T00:00:00Z`

#### Scenario: Location summary uses current simulation time

- **WHEN** a connected client sends the command `where` without an explicit timestamp
- **THEN** the server calculates the player ship location summary at the current authoritative simulation time

#### Scenario: Location summary avoids raw coordinates by default

- **WHEN** the server returns a location summary for `where`
- **THEN** the response does not include raw x/y/z coordinates by default

### Requirement: Fixed observer distance queries

The server SHALL represent the first-slice player ship as a spatial state in the ephemeris default frame and SHALL measure distances by deriving relative position from player ship and target state vectors.

#### Scenario: Report ship status

- **WHEN** the client requests status
- **THEN** the server response includes the current authoritative simulation time, ship id, ship name, ship motion mode, and ship frame

#### Scenario: Calculate distance from ship state

- **WHEN** a target object and the player ship both have resolved spatial states in a compatible frame
- **THEN** the server returns the Euclidean distance in kilometers and astronomical units derived from the relative state position

#### Scenario: Reject incompatible distance frames

- **WHEN** a target object and the player ship cannot be compared in a compatible frame
- **THEN** the server returns a clear query error instead of calculating a distance from unrelated coordinates

### Requirement: Server command handling

The server SHALL parse and handle the first-slice commands `help`, `objects`, `distance <object>`, `distance <object> --at <timestamp>`, `distances`, `distances --limit <n>`, `distances --sort distance`, `distances --at <timestamp>`, `status`, `time`, `advance <amount> <seconds|minutes|hours|days>`, `where`, `where <object>`, `where <object> --at <timestamp>`, `ship`, `ship status`, and `ship name <name>`.

#### Scenario: Handle objects command

- **WHEN** a connected client sends the command `objects`
- **THEN** the server responds with the known object list

#### Scenario: Handle distance command

- **WHEN** a connected client sends the command `distance mars`
- **THEN** the server responds with the distance result from the player ship to the object resolved from `mars`

#### Scenario: Handle distance command at explicit time

- **WHEN** a connected client sends the command `distance mars --at 2097-01-02T00:00:00Z`
- **THEN** the server responds with a distance result whose game time is `2097-01-02T00:00:00Z`

#### Scenario: Handle limited distances command

- **WHEN** a connected client sends the command `distances --limit 10`
- **THEN** the server responds with no more than 10 distance results

#### Scenario: Handle sorted distances command

- **WHEN** a connected client sends the command `distances --sort distance`
- **THEN** the server responds with distance results ordered by ascending distance from the player ship

#### Scenario: Handle where command

- **WHEN** a connected client sends the command `where`
- **THEN** the server responds with a location summary for the player ship

#### Scenario: Handle where object command

- **WHEN** a connected client sends the command `where mars`
- **THEN** the server responds with a location summary for the object resolved from `mars`

#### Scenario: Handle ship command

- **WHEN** a connected client sends the command `ship`
- **THEN** the server responds with the current player ship state

#### Scenario: Handle ship status command

- **WHEN** a connected client sends the command `ship status`
- **THEN** the server responds with the current player ship state

#### Scenario: Handle ship name command

- **WHEN** a connected client sends the command `ship name Wayfarer`
- **THEN** the server renames the player ship to `Wayfarer` and responds with updated player ship state

#### Scenario: Reject invalid ship name command

- **WHEN** a connected client sends `ship name` without a non-empty name
- **THEN** the server responds with a protocol error message that includes the command sequence number

#### Scenario: Handle unknown command

- **WHEN** a connected client sends a command the server does not support
- **THEN** the server responds with a protocol error message that includes the command sequence number

### Requirement: Server command completion

The server SHALL answer autocomplete requests using authoritative command metadata and runtime object data.

#### Scenario: Complete command name

- **WHEN** a connected client requests completion for input `di` with the cursor after `di`
- **THEN** the server responds with command-name candidates including `distance` and `distances`

#### Scenario: Complete where command

- **WHEN** a connected client requests completion for input `wh` with the cursor after `wh`
- **THEN** the server responds with the command-name candidate `where`

#### Scenario: Complete ship command

- **WHEN** a connected client requests completion for input `sh` with the cursor after `sh`
- **THEN** the server responds with the command-name candidate `ship`

#### Scenario: Complete object argument

- **WHEN** a connected client requests completion for input `distance ma` with the cursor after `ma`
- **THEN** the server responds with an object candidate for Mars using a replacement span that covers only `ma`

#### Scenario: Complete where object argument

- **WHEN** a connected client requests completion for input `where ma` with the cursor after `ma`
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

