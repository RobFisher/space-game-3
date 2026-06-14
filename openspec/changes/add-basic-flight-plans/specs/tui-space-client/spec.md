## ADDED Requirements

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
