## ADDED Requirements

### Requirement: Location summary presentation

The TUI client SHALL let users request the current observer location with `where` and SHALL present server location summaries as readable landmark-based text without showing raw coordinates by default.

#### Scenario: Submit where command

- **WHEN** the user enters `where`
- **THEN** the client sends a command message containing `where`

#### Scenario: Display location summary

- **WHEN** the client receives a location summary response
- **THEN** it appends a readable line containing the location label, nearest known object, distance, frame, and simulation time to the output log

#### Scenario: Hide raw coordinates by default

- **WHEN** the client displays a location summary response
- **THEN** the output does not include raw x/y/z coordinates by default

### Requirement: Plain text location output

Plain text mode SHALL print location summary responses as deterministic line-oriented text suitable for automated assertions.

#### Scenario: Print location summary response

- **WHEN** plain text mode receives a location summary response for a submitted `where` command
- **THEN** it prints a plain text line containing the location label, nearest known object, distance, frame, and simulation time
