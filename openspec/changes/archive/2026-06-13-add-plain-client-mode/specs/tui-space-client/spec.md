## ADDED Requirements

### Requirement: Plain text client mode

The TUI client binary SHALL provide a plain text mode that connects to the
configured server WebSocket endpoint without entering the terminal UI.

#### Scenario: Start plain mode

- **WHEN** the client binary is started with `--plain`
- **THEN** it connects to the server endpoint without entering the Ratatui
  alternate-screen interface

#### Scenario: Preserve default TUI mode

- **WHEN** the client binary is started without `--plain`
- **THEN** it starts the existing Ratatui interface

### Requirement: Plain text command input

Plain text mode SHALL send supported command text to the server using protocol
command messages with monotonically increasing sequence numbers.

#### Scenario: Send single command argument

- **WHEN** plain text mode is started with `--command "objects"`
- **THEN** it sends an `objects` command message to the server and exits after
  printing the command response

#### Scenario: Send stdin commands

- **WHEN** plain text mode receives newline-delimited commands on standard input
- **THEN** it sends each non-empty command to the server in order

#### Scenario: Exit from plain command input

- **WHEN** plain text mode receives `quit` or `exit` as command input
- **THEN** it exits without sending that command to the server

### Requirement: Plain text response output

Plain text mode SHALL print command responses and errors as deterministic
line-oriented text suitable for automated assertions.

#### Scenario: Print object response

- **WHEN** plain text mode receives an object list response for a submitted
  command
- **THEN** it prints a plain text line containing the known objects

#### Scenario: Print distance response

- **WHEN** plain text mode receives a distance response for a submitted command
- **THEN** it prints a plain text line containing the object name, kilometers,
  and astronomical units

#### Scenario: Print status response

- **WHEN** plain text mode receives a status response for a submitted `status`
  command
- **THEN** it prints plain text status values that can be asserted by tests

#### Scenario: Print error response

- **WHEN** plain text mode receives a protocol error for a submitted command
- **THEN** it prints a plain text error line containing the error code and
  message
