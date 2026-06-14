## ADDED Requirements

### Requirement: Ship protocol messages

The protocol SHALL represent server responses containing the current player ship id, display name, motion mode, frame, resolved game time, and optional spatial quality.

#### Scenario: Serialize ship state response

- **WHEN** a server ship response is serialized to JSON and deserialized again
- **THEN** the resulting message preserves the sequence number, ship id, ship name, motion mode, frame, game time, and quality

#### Scenario: Correlate ship response

- **WHEN** the server responds to a `ship` command with sequence number 11
- **THEN** the ship response includes sequence number 11

## MODIFIED Requirements

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

