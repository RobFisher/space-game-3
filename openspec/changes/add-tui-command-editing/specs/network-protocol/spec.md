## ADDED Requirements

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
