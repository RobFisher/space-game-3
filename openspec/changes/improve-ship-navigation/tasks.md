## 1. Navigation Module

- [x] 1.1 Add a dedicated server navigation module and wire it into the server crate module tree.
- [x] 1.2 Implement acceleration parsing/conversion helpers for km/s² values and `g`-suffixed values.
- [x] 1.3 Add navigation profile types for maximum acceleration and optional specific impulse validation.
- [x] 1.4 Add arrival orbit request and resolved orbit estimate types for default, low, stationary, and custom circular orbits.
- [x] 1.5 Implement circular orbit estimate calculations for radius, altitude, period, and circular speed when required constants are available.
- [x] 1.6 Add unit tests for acceleration conversion, invalid engine/profile values, orbit preset resolution, and orbital estimate calculations.

## 2. Flight Plan Model and State Resolution

- [x] 2.1 Extend the flight plan model with arrival orbit request, resolved orbit estimate, transfer arrival time, orbit entry completion time, navigation phase, and acceleration-in-G display metadata.
- [x] 2.2 Move transfer duration/interpolation and intercept planning into the navigation module while preserving deterministic behavior.
- [x] 2.3 Change object-target planning to snapshot an orbital insertion state on the requested arrival orbit instead of the destination object center.
- [x] 2.4 Implement `entering_orbit` state resolution that blends from transfer endpoint to circular orbit state.
- [x] 2.5 Update player ship state resolution to report `flight_plan`, `entering_orbit`, and `orbiting` phases at the correct times.
- [x] 2.6 Add focused tests for insertion-state planning, explicit-time queries, orbit-entry resolution, and completed handoff to orbiting motion.

## 3. Protocol and Server Commands

- [x] 3.1 Extend protocol DTOs with navigation phase, transfer arrival time, orbit entry completion time, acceleration in G, and arrival orbit estimate fields.
- [x] 3.2 Update protocol serialization tests for the extended flight plan and entering-orbit ship state messages.
- [x] 3.3 Extend server flight command parsing for `--accel 0.5g`, `--orbit`, `--orbit-altitude`, and `--orbit-radius`.
- [x] 3.4 Add clear server errors for invalid acceleration, invalid custom orbit values, and unsupported stationary orbit requests.
- [x] 3.5 Update server command/query tests for G acceleration, orbit presets, custom orbit requests, and flight-plan-aware queries during orbit entry.

## 4. TUI and Plain Client

- [x] 4.1 Update command help and completion candidates for G acceleration and orbit options.
- [x] 4.2 Update interactive flight plan output to show navigation phase, acceleration in G, orbit entry completion time, and arrival orbit summary.
- [x] 4.3 Update plain text flight plan output with deterministic navigation/orbit fields for assertions.
- [x] 4.4 Update the status pane to show navigation phase and arrival orbit period when known.
- [x] 4.5 Add or update client tests for plain output and command handling.

## 5. Validation and Documentation

- [x] 5.1 Update README or user-facing command documentation for G acceleration and orbit options.
- [x] 5.2 Run relevant Rust tests for `space-server`, `space-game-protocol`, and `space-client-tui`.
- [x] 5.3 Run `openspec validate --all` and resolve any validation issues.
- [x] 5.4 Review the change artifacts against the final implementation and update tasks/specs if behavior changes during implementation.

## 6. Flight Display Refinement

- [x] 6.1 Update active transfer display to show current planned acceleration and transfer speed instead of arrival orbit details.
- [x] 6.2 Update entering-orbit display to show an entering-orbit indicator instead of arrival orbit details.
- [x] 6.3 Keep arrival orbit details visible once the flight plan has reached orbiting phase.
- [x] 6.4 Run affected client tests and `openspec validate --all`.
