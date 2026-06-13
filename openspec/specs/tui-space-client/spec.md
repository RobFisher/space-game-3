# TUI Space Client Specification

## Purpose

Define the Ratatui client that connects to the local authoritative server, sends user commands, and presents output, status, and errors for the first networked space-game slice.

## Requirements

### Requirement: TUI connection lifecycle

The TUI client SHALL connect to the configured server WebSocket endpoint, maintain connection state in its view model, and restore the terminal when exiting.

#### Scenario: Connect at startup

- **WHEN** the TUI client starts with default configuration and the local server is available
- **THEN** it connects to the default server endpoint and displays connected status

#### Scenario: Exit cleanly

- **WHEN** the user enters `quit` or `exit`
- **THEN** the client exits its event loop and restores the terminal

### Requirement: OpenCode-style layout

The TUI client SHALL render a structured layout with an output log, a status pane, and a command input area.

#### Scenario: Render primary panes

- **WHEN** the TUI client draws a frame
- **THEN** the frame contains an output log region, a status region, and a command input region

#### Scenario: Update status independently

- **WHEN** a status message is received while the user is editing command input
- **THEN** the status pane updates without clearing the command input

### Requirement: Command submission

The TUI client SHALL send supported user-entered command text to the server using protocol command messages with monotonically increasing sequence numbers.

#### Scenario: Submit objects command

- **WHEN** the user enters `objects`
- **THEN** the client sends a command message containing `objects`

#### Scenario: Submit distance command

- **WHEN** the user enters `distance mars`
- **THEN** the client sends a command message containing `distance mars`

### Requirement: Advancing clock display

The TUI client SHALL display an advancing simulation clock based on the latest server time sample.

#### Scenario: Display projected clock

- **WHEN** the client has received a server simulation time sample with running state enabled
- **THEN** the status pane displays a simulation clock that advances as the TUI redraws

#### Scenario: Resync displayed clock

- **WHEN** the client receives a newer server simulation time or status message
- **THEN** the client updates its local clock sample from the server-provided timestamp

### Requirement: Simulation time commands

The TUI client SHALL let users request and advance simulation time through command input.

#### Scenario: Submit time command

- **WHEN** the user enters `time`
- **THEN** the client sends a command message containing `time`

#### Scenario: Submit advance command

- **WHEN** the user enters `advance 10 minutes`
- **THEN** the client sends a command message containing `advance 10 minutes`

### Requirement: Server message presentation

The TUI client SHALL update its local view model from server protocol messages and present object lists, distance results, errors, status values, and simulation time values to the user.

#### Scenario: Display object list

- **WHEN** the client receives an object list response
- **THEN** it appends a readable object list entry to the output log

#### Scenario: Display single distance

- **WHEN** the client receives a distance response
- **THEN** it appends a readable line containing the object name, kilometers, and astronomical units to the output log

#### Scenario: Display distance list

- **WHEN** the client receives a distances response
- **THEN** it appends readable distance entries to the output log

#### Scenario: Display simulation time

- **WHEN** the client receives a simulation time response for a submitted command
- **THEN** it appends a readable line containing the current simulation timestamp to the output log

#### Scenario: Display protocol error

- **WHEN** the client receives an error response
- **THEN** it appends the error message to the output log without exiting unless the user explicitly quits

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

#### Scenario: Print simulation time response

- **WHEN** plain text mode receives a simulation time response for a submitted `time` or `advance` command
- **THEN** it prints a plain text line containing the current simulation timestamp

#### Scenario: Print error response

- **WHEN** plain text mode receives a protocol error for a submitted command
- **THEN** it prints a plain text error line containing the error code and
  message
