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

#### Scenario: Show ship name in status pane

- **WHEN** the TUI client draws a frame after receiving server status
- **THEN** the status pane shows the current player ship name

### Requirement: Command submission

The TUI client SHALL send supported user-entered command text to the server using protocol command messages with monotonically increasing sequence numbers.

#### Scenario: Submit objects command

- **WHEN** the user enters `objects`
- **THEN** the client sends a command message containing `objects`

#### Scenario: Submit distance command

- **WHEN** the user enters `distance mars`
- **THEN** the client sends a command message containing `distance mars`

### Requirement: Flight plan command UX

The TUI client SHALL let users create, inspect, and cancel player ship flight plans by submitting flight commands to the server.

#### Scenario: Submit flight plan command

- **WHEN** the user enters `flight plan mars --accel 0.02`
- **THEN** the client sends a command message containing `flight plan mars --accel 0.02`

#### Scenario: Submit flight status command

- **WHEN** the user enters `flight status`
- **THEN** the client sends a command message containing `flight status`

#### Scenario: Submit flight cancel command

- **WHEN** the user enters `flight cancel`
- **THEN** the client sends a command message containing `flight cancel`

### Requirement: Location summary presentation

The TUI client SHALL let users request the current player ship location with `where`, request object locations with `where <object> [--at <timestamp>]`, and SHALL present server location summaries as readable landmark-based text without showing raw coordinates by default.

#### Scenario: Submit where command

- **WHEN** the user enters `where`
- **THEN** the client sends a command message containing `where`

#### Scenario: Submit where object command

- **WHEN** the user enters `where mars --at 2097-01-02T00:00:00Z`
- **THEN** the client sends a command message containing `where mars --at 2097-01-02T00:00:00Z`

#### Scenario: Display location summary

- **WHEN** the client receives a location summary response
- **THEN** it appends a readable line containing the location label, nearest known object, distance, frame, and simulation time to the output log

#### Scenario: Hide raw coordinates by default

- **WHEN** the client displays a location summary response
- **THEN** the output does not include raw x/y/z coordinates by default

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

The TUI client SHALL update its local view model from server protocol messages and present object lists, distance results, errors, status values, ship state values, and simulation time values to the user.

#### Scenario: Display object list

- **WHEN** the client receives an object list response
- **THEN** it appends a readable object list entry to the output log

#### Scenario: Display single distance

- **WHEN** the client receives a distance response
- **THEN** it appends a readable line containing the object name, kilometers, and astronomical units to the output log

#### Scenario: Display distance list

- **WHEN** the client receives a distances response
- **THEN** it appends readable distance entries to the output log

#### Scenario: Display ship state

- **WHEN** the client receives a ship state response
- **THEN** it appends a readable line containing the ship name, motion mode, frame, and simulation time to the output log

#### Scenario: Display simulation time

- **WHEN** the client receives a simulation time response for a submitted command
- **THEN** it appends a readable line containing the current simulation timestamp to the output log

#### Scenario: Display protocol error

- **WHEN** the client receives an error response
- **THEN** it appends the error message to the output log without exiting unless the user explicitly quits

### Requirement: Flight plan presentation

The TUI client SHALL present server flight plan responses as readable gameplay text.

#### Scenario: Display active flight plan

- **WHEN** the client receives an active flight plan response
- **THEN** it appends a readable line containing the target, status, acceleration, departure time, arrival time, and duration to the output log

#### Scenario: Display no active flight plan

- **WHEN** the client receives a flight plan response indicating no active plan
- **THEN** it appends a readable no-active-plan line to the output log

#### Scenario: Plain mode displays flight plan response

- **WHEN** plain text mode receives a flight plan response for a submitted flight command
- **THEN** it prints a plain text line containing the target, status, acceleration, departure time, arrival time, and duration

#### Scenario: Status pane displays active flight ETA

- **WHEN** the interactive TUI has received an active flight plan response
- **THEN** the status pane displays the active flight target, arrival time, and countdown to arrival derived from the current displayed game time

#### Scenario: Status pane displays destination distance

- **WHEN** the interactive TUI has an active flight plan with duration and acceleration
- **THEN** the status pane displays the estimated remaining distance to the destination derived from the flight plan timing and current displayed game time

#### Scenario: Startup syncs active flight status

- **WHEN** the interactive TUI connects to the server
- **THEN** it requests current flight status without adding a user-visible command response to the output log

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
- **THEN** it prints plain text status values including the player ship name that can be asserted by tests

#### Scenario: Print ship response

- **WHEN** plain text mode receives a ship state response for a submitted `ship` command
- **THEN** it prints a plain text line containing the ship name, motion mode, frame, and simulation time

#### Scenario: Print simulation time response

- **WHEN** plain text mode receives a simulation time response for a submitted `time` or `advance` command
- **THEN** it prints a plain text line containing the current simulation timestamp

#### Scenario: Print error response

- **WHEN** plain text mode receives a protocol error for a submitted command
- **THEN** it prints a plain text error line containing the error code and
  message

### Requirement: Plain text location output

Plain text mode SHALL print location summary responses as deterministic line-oriented text suitable for automated assertions.

#### Scenario: Print location summary response

- **WHEN** plain text mode receives a location summary response for a submitted `where` command
- **THEN** it prints a plain text line containing the location label, nearest known object, distance, frame, and simulation time

### Requirement: Readline-style command history

The TUI client SHALL maintain command history that can be browsed with Up and Down arrows, and recalled history entries SHALL remain editable before submission.

#### Scenario: Browse previous command

- **WHEN** the user has submitted `objects` and then presses Up from an empty command input
- **THEN** the command input contains `objects` with the cursor positioned for editing

#### Scenario: Edit recalled command

- **WHEN** the user recalls `distance mars`, edits it to `distance luna`, and presses Enter
- **THEN** the client sends a command message containing `distance luna`

#### Scenario: Restore draft while browsing history

- **WHEN** the user has typed an unsent draft, presses Up to browse history, and presses Down back past the newest history entry
- **THEN** the command input restores the unsent draft

### Requirement: Persistent command history

The TUI client SHALL persist submitted command history to a local user data file and load that history when a later TUI session starts.

#### Scenario: Save submitted command

- **WHEN** the user submits a non-empty command other than `quit` or `exit`
- **THEN** the command is saved to the local command history store

#### Scenario: Load command history

- **WHEN** the TUI client starts after a previous session saved `status`
- **THEN** pressing Up can recall `status`

#### Scenario: Bound history size

- **WHEN** saved command history exceeds the configured maximum history length
- **THEN** the client retains only the newest commands up to that limit

#### Scenario: Isolate non-history tests

- **WHEN** tests exercise command input behavior other than history persistence
- **THEN** the client uses injected, temporary, in-memory, or disabled history storage instead of the real user history file

### Requirement: Reverse history search

The TUI client SHALL provide a Ctrl-R reverse history search mode that incrementally searches previously submitted commands.

#### Scenario: Open reverse search

- **WHEN** the user presses Ctrl-R while editing command input
- **THEN** the client enters reverse history search mode without submitting a command

#### Scenario: Incremental search updates match

- **WHEN** the user types search text while in reverse history search mode
- **THEN** the client displays the newest history entry that contains the search text

#### Scenario: Accept reverse search match

- **WHEN** reverse history search has a current match and the user presses Enter
- **THEN** the client places that match into normal command input for editing or submission

#### Scenario: Cancel reverse search

- **WHEN** the user presses Esc while in reverse history search mode
- **THEN** the client exits reverse history search and restores the command draft from before search began

### Requirement: Server-backed command completion UX

The TUI client SHALL request autocomplete candidates from the server when the user presses Tab and SHALL apply or display the server-provided completion response without requiring cached object lists.

#### Scenario: Request completion

- **WHEN** the user presses Tab with command input `distance ma`
- **THEN** the client sends a completion request containing the full input text and current cursor position

#### Scenario: Complete local-only command

- **WHEN** the user presses Tab with command input `qu`
- **THEN** the client can complete the local-only command `quit` without requiring the server to execute that command

#### Scenario: Apply single completion

- **WHEN** the client receives one completion candidate with a replacement span for the active completion request
- **THEN** the client replaces that span with the candidate insertion text and moves the cursor to the end of the inserted text

#### Scenario: Apply longest common prefix

- **WHEN** the client receives multiple completion candidates that share a longer insertion prefix than the current replacement text
- **THEN** the client replaces the active span with the longest common prefix before displaying the remaining candidates

#### Scenario: Display multiple completions

- **WHEN** the client receives multiple completion candidates for the active completion request
- **THEN** the TUI displays the candidates without submitting a command

#### Scenario: Show delayed completion spinner

- **WHEN** a completion request has been pending for more than 0.2 seconds
- **THEN** the TUI displays a spinner or pending indicator for autocomplete

#### Scenario: Suppress fast completion spinner

- **WHEN** a completion request completes before it has been pending for 0.2 seconds
- **THEN** the TUI does not display the autocomplete spinner

#### Scenario: Cancel pending completion

- **WHEN** a completion request is pending and the user presses Esc
- **THEN** the client cancels the pending completion locally and ignores any later response for that request
