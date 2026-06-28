## MODIFIED Requirements

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

## ADDED Requirements

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
