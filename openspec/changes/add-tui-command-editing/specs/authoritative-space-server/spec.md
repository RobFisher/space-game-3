## ADDED Requirements

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
