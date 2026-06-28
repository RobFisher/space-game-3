## MODIFIED Requirements

### Requirement: Server command handling

The server SHALL parse and handle the first-slice commands `help`, `objects`, `distance <object>`, `distance <object> --at <timestamp>`, `distances`, `distances --limit <n>`, `distances --sort distance`, `distances --at <timestamp>`, `status`, `time`, `advance <amount> <seconds|minutes|hours|days>`, `where`, `where <object>`, `where <object> --at <timestamp>`, `ship`, `ship status`, `ship name <name>`, `flight plan <object> [--accel <km_per_s2|g>] [--orbit <default|low|stationary>] [--orbit-altitude <km>] [--orbit-radius <km>]`, `flight status`, and `flight cancel`.

#### Scenario: Handle objects command

- **WHEN** a connected client sends the command `objects`
- **THEN** the server responds with the known object list

#### Scenario: Handle distance command

- **WHEN** a connected client sends the command `distance mars`
- **THEN** the server responds with the distance result from the player ship to the object resolved from `mars`

#### Scenario: Handle distance command at explicit time

- **WHEN** a connected client sends the command `distance mars --at 2097-01-02T00:00:00Z`
- **THEN** the server responds with a distance result whose game time is `2097-01-02T00:00:00Z`

#### Scenario: Handle limited distances command

- **WHEN** a connected client sends the command `distances --limit 10`
- **THEN** the server responds with no more than 10 distance results

#### Scenario: Handle sorted distances command

- **WHEN** a connected client sends the command `distances --sort distance`
- **THEN** the server responds with distance results ordered by ascending distance from the player ship

#### Scenario: Handle where command

- **WHEN** a connected client sends the command `where`
- **THEN** the server responds with a location summary for the player ship

#### Scenario: Handle where object command

- **WHEN** a connected client sends the command `where mars`
- **THEN** the server responds with a location summary for the object resolved from `mars`

#### Scenario: Handle ship command

- **WHEN** a connected client sends the command `ship`
- **THEN** the server responds with the current player ship state

#### Scenario: Handle ship status command

- **WHEN** a connected client sends the command `ship status`
- **THEN** the server responds with the current player ship state

#### Scenario: Handle ship name command

- **WHEN** a connected client sends the command `ship name Wayfarer`
- **THEN** the server renames the player ship to `Wayfarer` and responds with updated player ship state

#### Scenario: Reject invalid ship name command

- **WHEN** a connected client sends `ship name` without a non-empty name
- **THEN** the server responds with a protocol error message that includes the command sequence number

#### Scenario: Handle flight plan command

- **WHEN** a connected client sends the command `flight plan mars --accel 0.02`
- **THEN** the server creates a flight plan from the current player ship state to the object resolved from `mars` using acceleration `0.02` kilometers per second squared and responds with the flight plan

#### Scenario: Handle flight plan command with G acceleration

- **WHEN** a connected client sends the command `flight plan mars --accel 0.5g`
- **THEN** the server creates a flight plan using acceleration converted from `0.5` standard gravity and responds with the flight plan

#### Scenario: Handle flight plan command with arrival orbit

- **WHEN** a connected client sends the command `flight plan mars --orbit low`
- **THEN** the server creates a flight plan targeting a low circular arrival orbit around the object resolved from `mars`

#### Scenario: Handle flight plan command with default acceleration

- **WHEN** a connected client sends the command `flight plan mars`
- **THEN** the server creates a flight plan using the configured default acceleration and responds with the flight plan

#### Scenario: Handle flight status command

- **WHEN** a connected client sends the command `flight status`
- **THEN** the server responds with the current active flight plan or a clear no-active-plan response

#### Scenario: Handle flight cancel command

- **WHEN** a connected client sends the command `flight cancel`
- **THEN** the server cancels the current active flight plan and responds with the cancelled flight plan status

#### Scenario: Handle unknown command

- **WHEN** a connected client sends a command the server does not support
- **THEN** the server responds with a protocol error message that includes the command sequence number

### Requirement: Flight plan-aware queries

The server SHALL use active flight plan and orbit-entry motion when resolving player ship state for status, ship, distance, distances, and where queries.

#### Scenario: Distance uses active flight plan

- **WHEN** a connected client requests distance to an object while the player ship has an active flight plan
- **THEN** the server calculates the distance from the ship position resolved from that flight plan at the effective simulation time

#### Scenario: Ship status reports flight plan motion

- **WHEN** a connected client requests ship status during an active flight plan transfer
- **THEN** the server response includes a ship motion mode indicating flight plan motion

#### Scenario: Ship status reports entering orbit motion

- **WHEN** a connected client requests ship status during an orbit-entry phase
- **THEN** the server response includes a ship motion mode indicating entering orbit motion
