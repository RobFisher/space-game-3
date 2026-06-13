## 1. Protocol Contracts

- [x] 1.1 Add protocol DTOs for simulation time state, time units, simulation time requests, and simulation time advancement requests.
- [x] 1.2 Add optional explicit timestamp fields to distance and distances request DTOs while preserving backward-compatible deserialization.
- [x] 1.3 Add protocol serialization round-trip tests for simulation time messages, time advancement, and explicit distance timestamps.

## 2. Server Clock Model

- [x] 2.1 Add a deterministic server simulation clock model initialized from `DEFAULT_GAME_TIME`.
- [x] 2.2 Implement current-time calculation from an anchor simulation time, anchor wall-clock instant, running state, and fixed rate.
- [x] 2.3 Implement manual advancement by seconds, minutes, hours, and days with unit validation and deterministic tests.

## 3. Server Command and WebSocket Handling

- [x] 3.1 Store the simulation clock in server application state alongside the existing query service.
- [x] 3.2 Update status and welcome-time behavior to use the current simulation clock rather than reparsing the fixed default timestamp.
- [x] 3.3 Handle typed simulation time query and advancement protocol messages.
- [x] 3.4 Add `time` and `advance <amount> <seconds|minutes|hours|days>` command handling with success and error responses.
- [x] 3.5 Update distance and distances handling so omitted timestamps use the current simulation clock and explicit timestamps do not mutate it.
- [x] 3.6 Add server unit and WebSocket tests covering current time, manual advancement, explicit distance timestamps, and default clock-based distance timestamps.

## 4. TUI Client Experience

- [ ] 4.1 Extend client view state to store the latest server simulation time sample, running state, rate, and local receipt instant.
- [ ] 4.2 Render a projected advancing simulation clock in the status pane during normal TUI redraws.
- [ ] 4.3 Periodically request simulation time from the server to resync the displayed clock.
- [ ] 4.4 Display simulation time responses in the output log for `time` and `advance` command responses.
- [ ] 4.5 Update plain text mode formatting and command completion handling for simulation time responses.
- [ ] 4.6 Add TUI and plain-mode tests for clock sample updates, projected display time, and simulation time response output.

## 5. Documentation and Validation

- [ ] 5.1 Update README or command help text to mention `time`, `advance`, and optional `--at` distance timestamps.
- [ ] 5.2 Run relevant Rust tests for protocol, server, ephemeris time behavior, and TUI client changes.
- [ ] 5.3 Run `openspec validate --all` and resolve any validation issues.
