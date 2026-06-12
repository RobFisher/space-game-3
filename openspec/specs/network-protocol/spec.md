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

#### Scenario: Correlate error response

- **WHEN** the server rejects a command associated with sequence number 8
- **THEN** the error response includes sequence number 8

### Requirement: Status messages

The protocol SHALL represent server status messages containing game time, observer label, observer frame, object count, and connection-relevant server information.

#### Scenario: Receive unsolicited status

- **WHEN** the server sends a status update that is not a direct response to a command
- **THEN** the message can omit a request sequence number while preserving status fields
