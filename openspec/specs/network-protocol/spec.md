# Network Protocol Specification

## Purpose

Define the shared JSON-serializable client/server protocol used by the first networked TUI slice.

## Requirements

### Requirement: Serializable WebSocket protocol

The system SHALL define shared client-to-server and server-to-client message types that can be serialized to and deserialized from JSON for WebSocket transport.

#### Scenario: Serialize client command

- **WHEN** a client command message with a sequence number and command text is serialized to JSON and deserialized again
- **THEN** the resulting message preserves the original sequence number and command text

#### Scenario: Serialize server distance response

- **WHEN** a server distance response containing object identity, display name, distance kilometers, distance astronomical units, and game time is serialized to JSON and deserialized again
- **THEN** the resulting message preserves all distance result fields

#### Scenario: Serialize server ship response

- **WHEN** a server ship response containing ship identity, display name, motion mode, frame, game time, and quality is serialized to JSON and deserialized again
- **THEN** the resulting message preserves all ship result fields

### Requirement: Ship protocol messages

The protocol SHALL represent server responses containing the current player ship id, display name, motion mode, frame, resolved game time, and optional spatial quality.

#### Scenario: Serialize ship state response

- **WHEN** a server ship response is serialized to JSON and deserialized again
- **THEN** the resulting message preserves the sequence number, ship id, ship name, motion mode, frame, game time, and quality

#### Scenario: Correlate ship response

- **WHEN** the server responds to a `ship` command with sequence number 11
- **THEN** the ship response includes sequence number 11

### Requirement: Location summary protocol messages

The protocol SHALL represent server location summary responses containing the subject identity/label, subject type, frame, game time, nearest known object identity, nearest known object display name, distance kilometers, distance astronomical units, and optional spatial quality.

#### Scenario: Serialize location summary response

- **WHEN** a server location summary response is serialized to JSON and deserialized again
- **THEN** the resulting message preserves the sequence number, subject fields, frame, game time, nearest object fields, distance fields, and quality

#### Scenario: Location summary omits raw coordinates

- **WHEN** a location summary response is serialized
- **THEN** the response does not include raw x/y/z coordinate fields by default

#### Scenario: Correlate location summary response

- **WHEN** the server responds to a `where` command with sequence number 9
- **THEN** the location summary response includes sequence number 9

### Requirement: Protocol DTOs are UI and ephemeris independent

The protocol crate SHALL define wire-visible DTOs without depending on Ratatui, Crossterm, Axum, Tungstenite, or the server implementation.

#### Scenario: Build protocol crate independently

- **WHEN** the protocol crate is built
- **THEN** it compiles without requiring TUI, WebSocket server, or ephemeris implementation dependencies

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

The protocol SHALL represent server status messages containing game time, player ship id, player ship name, player ship frame, player ship motion mode, object count, and connection-relevant server information.

#### Scenario: Receive unsolicited status

- **WHEN** the server sends a status update that is not a direct response to a command
- **THEN** the message can omit a request sequence number while preserving status fields

#### Scenario: Receive status with authoritative simulation time

- **WHEN** the server sends a status update
- **THEN** the status message includes the current authoritative simulation time

#### Scenario: Receive status with ship identity

- **WHEN** the server sends a status update
- **THEN** the status message includes the current player ship id, player ship name, player ship frame, and player ship motion mode

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

### Requirement: Completion protocol messages

The protocol SHALL represent client autocomplete requests and server autocomplete responses with sequence correlation, command input text, cursor position, replacement span, and typed completion candidates.

#### Scenario: Serialize completion request

- **WHEN** a completion request containing sequence number `12`, input text `distance ma`, and cursor position `11` is serialized to JSON and deserialized again
- **THEN** the resulting message preserves the sequence number, input text, and cursor position

#### Scenario: Serialize completion response

- **WHEN** a completion response containing sequence number `12`, a replacement span, and an object candidate for `mars` is serialized to JSON and deserialized again
- **THEN** the resulting message preserves the sequence number, replacement span, candidate insertion text, candidate display text, and candidate kind

#### Scenario: Correlate completion response

- **WHEN** the server responds to a completion request with sequence number `15`
- **THEN** the completion response includes sequence number `15`

#### Scenario: Represent no completions

- **WHEN** the server has no autocomplete candidates for a request
- **THEN** the completion response can represent an empty candidate list while preserving the request sequence number

### Requirement: Flight plan protocol messages

The protocol SHALL represent server flight plan responses containing plan id, ship id, target information, departure time, transfer arrival time, orbit entry completion time, duration seconds, acceleration in simulation units, optional acceleration in G, status, navigation phase, arrival orbit details, and optional spatial quality.

#### Scenario: Serialize flight plan response

- **WHEN** a server flight plan response is serialized to JSON and deserialized again
- **THEN** the resulting message preserves the sequence number, plan id, ship id, target fields, departure time, transfer arrival time, orbit entry completion time, duration seconds, acceleration values, status, navigation phase, arrival orbit fields, and quality

#### Scenario: Correlate flight plan response

- **WHEN** the server responds to a `flight plan mars` command with sequence number 14
- **THEN** the flight plan response includes sequence number 14

#### Scenario: Represent no active flight plan

- **WHEN** the server responds to a `flight status` command and no flight plan is active
- **THEN** the protocol response can represent the absence of an active flight plan while preserving the request sequence number

### Requirement: Navigation motion modes
The protocol SHALL represent player ship motion modes for orbiting, flight plan transfer, and entering orbit states.

#### Scenario: Serialize entering orbit ship state
- **WHEN** a ship state response reports `entering_orbit` motion
- **THEN** the serialized protocol message preserves that motion mode

### Requirement: Arrival orbit protocol fields
The protocol SHALL represent flight plan arrival orbit estimates with orbit kind, radius, altitude, period, and circular speed fields when available.

#### Scenario: Serialize arrival orbit estimate
- **WHEN** a flight plan response includes a resolved arrival orbit estimate
- **THEN** the serialized protocol message preserves the orbit kind, radius kilometers, altitude kilometers, period seconds when known, and circular speed kilometers per second when known
