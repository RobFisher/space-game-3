## MODIFIED Requirements

### Requirement: Flight plan command UX

The TUI client SHALL let users create, inspect, and cancel player ship flight plans by submitting flight commands with acceleration and arrival orbit options to the server.

#### Scenario: Submit flight plan command

- **WHEN** the user enters `flight plan mars --accel 0.02`
- **THEN** the client sends a command message containing `flight plan mars --accel 0.02`

#### Scenario: Submit flight plan command with G acceleration

- **WHEN** the user enters `flight plan mars --accel 0.5g`
- **THEN** the client sends a command message containing `flight plan mars --accel 0.5g`

#### Scenario: Submit flight plan command with orbit option

- **WHEN** the user enters `flight plan mars --orbit low`
- **THEN** the client sends a command message containing `flight plan mars --orbit low`

#### Scenario: Submit flight status command

- **WHEN** the user enters `flight status`
- **THEN** the client sends a command message containing `flight status`

#### Scenario: Submit flight cancel command

- **WHEN** the user enters `flight cancel`
- **THEN** the client sends a command message containing `flight cancel`

### Requirement: Flight plan presentation

The TUI client SHALL present server flight plan responses as readable gameplay text including navigation phase, phase-appropriate navigation details, acceleration, timing, and arrival orbit estimates.

#### Scenario: Display active flight plan

- **WHEN** the client receives an active flight plan response
- **THEN** it appends a readable line containing the target, status, navigation phase, acceleration, departure time, arrival time, orbit entry completion time, duration, and phase-appropriate flight or orbit summary to the output log

#### Scenario: Display no active flight plan

- **WHEN** the client receives a flight plan response indicating no active plan
- **THEN** it appends a readable no-active-plan line to the output log

#### Scenario: Plain mode displays flight plan response

- **WHEN** plain text mode receives a flight plan response for a submitted flight command
- **THEN** it prints a plain text line containing the target, status, navigation phase, acceleration, departure time, arrival time, orbit entry completion time, duration, and phase-appropriate flight or orbit summary

#### Scenario: Status pane displays active flight ETA

- **WHEN** the interactive TUI has received an active flight plan response
- **THEN** the status pane displays the active flight target, arrival time, navigation phase, and countdown to arrival derived from the current displayed game time

#### Scenario: Status pane displays active transfer dynamics

- **WHEN** the interactive TUI has an active flight plan in flight-plan transfer phase
- **THEN** the status pane displays current planned acceleration and transfer speed rather than arrival orbit details

#### Scenario: Status pane displays orbit entry

- **WHEN** the interactive TUI has an active flight plan in entering-orbit phase
- **THEN** the status pane displays an entering-orbit indicator rather than arrival orbit details

#### Scenario: Status pane displays destination distance

- **WHEN** the interactive TUI has an active flight plan with duration and acceleration
- **THEN** the status pane displays the estimated remaining distance to the destination derived from the flight plan timing and current displayed game time

#### Scenario: Status pane displays arrival orbit

- **WHEN** the interactive TUI has an active flight plan in orbiting phase with a resolved arrival orbit estimate
- **THEN** the status pane displays the arrival orbit kind and orbital period when known

#### Scenario: Startup syncs active flight status

- **WHEN** the interactive TUI connects to the server
- **THEN** it requests current flight status without adding a user-visible command response to the output log
