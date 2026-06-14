## ADDED Requirements

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
