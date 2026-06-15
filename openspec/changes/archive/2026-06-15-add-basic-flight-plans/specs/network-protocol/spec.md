## ADDED Requirements

### Requirement: Flight plan protocol messages
The protocol SHALL represent server flight plan responses containing plan id, ship id, target information, departure time, arrival time, duration seconds, acceleration, status, and optional spatial quality.

#### Scenario: Serialize flight plan response
- **WHEN** a server flight plan response is serialized to JSON and deserialized again
- **THEN** the resulting message preserves the sequence number, plan id, ship id, target fields, departure time, arrival time, duration seconds, acceleration, status, and quality

#### Scenario: Correlate flight plan response
- **WHEN** the server responds to a `flight plan mars` command with sequence number 14
- **THEN** the flight plan response includes sequence number 14

#### Scenario: Represent no active flight plan
- **WHEN** the server responds to a `flight status` command and no flight plan is active
- **THEN** the protocol response can represent the absence of an active flight plan while preserving the request sequence number
