## MODIFIED Requirements

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

### Requirement: Plain text response output

Plain text mode SHALL print command responses and errors as deterministic line-oriented text suitable for automated assertions.

#### Scenario: Print object response

- **WHEN** plain text mode receives an object list response for a submitted command
- **THEN** it prints a plain text line containing the known objects

#### Scenario: Print distance response

- **WHEN** plain text mode receives a distance response for a submitted command
- **THEN** it prints a plain text line containing the object name, kilometers, and astronomical units

#### Scenario: Print status response

- **WHEN** plain text mode receives a status response for a submitted `status` command
- **THEN** it prints plain text status values including the player ship name that can be asserted by tests

#### Scenario: Print ship response

- **WHEN** plain text mode receives a ship state response for a submitted `ship` command
- **THEN** it prints a plain text line containing the ship name, motion mode, frame, and simulation time

#### Scenario: Print simulation time response

- **WHEN** plain text mode receives a simulation time response for a submitted `time` or `advance` command
- **THEN** it prints a plain text line containing the current simulation timestamp

#### Scenario: Print error response

- **WHEN** plain text mode receives a protocol error for a submitted command
- **THEN** it prints a plain text error line containing the error code and message

