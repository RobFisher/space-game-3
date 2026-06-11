## 1. Workspace and Protocol

- [ ] 1.1 Add `space-game-protocol`, `space-server`, and `space-client-tui` crates to the Cargo workspace.
- [ ] 1.2 Define protocol message enums for client-to-server and server-to-client JSON messages.
- [ ] 1.3 Define protocol DTOs for object summaries, distance results, status, sort options, and displayable errors.
- [ ] 1.4 Add protocol serialization round-trip tests for command, status, object list, distance, and error messages.

## 2. Server Query Core

- [ ] 2.1 Add a fictional demo registry TOML file under the server crate using supported ephemeris sources only.
- [ ] 2.2 Implement server startup configuration for the default bind address, WebSocket path, demo registry, and fixed observer.
- [ ] 2.3 Implement a `SolarSystemQueryService` that lists objects, resolves object queries, reports status, and calculates distances from the fixed observer.
- [ ] 2.4 Add query service tests for object listing, lowercase id lookup, display-name lookup, ambiguous lookup errors, single distance, sorted distances, and limited distances.
- [ ] 2.5 Implement command parsing and handling for `help`, `objects`, `distance <object>`, `distances`, `distances --limit <n>`, `distances --sort distance`, `status`, and unknown commands.
- [ ] 2.6 Add command handler tests that verify protocol responses and sequence-number correlation.

## 3. Server WebSocket

- [ ] 3.1 Implement the local Axum WebSocket endpoint at `/ws`.
- [ ] 3.2 Send welcome and initial status messages when a client connects.
- [ ] 3.3 Deserialize incoming protocol messages, route them through the command/query handlers, and serialize outgoing protocol messages.
- [ ] 3.4 Add integration coverage for a client connection that requests objects, a single distance, multiple distances, and status.

## 4. TUI Client

- [ ] 4.1 Implement the TUI application view model for connection state, server URL, output lines, status, command input, sequence numbers, and quit state.
- [ ] 4.2 Implement terminal setup and restoration for Ratatui/Crossterm.
- [ ] 4.3 Render the output log, status pane, and command input area in a stable OpenCode-style layout.
- [ ] 4.4 Implement keyboard handling for text input, command submission, backspace/editing basics, and `quit`/`exit`.
- [ ] 4.5 Implement the WebSocket client connection and async event loop that merges terminal events, network messages, and render ticks.
- [ ] 4.6 Implement server message application for welcome, status, object list, distance, distances, output lines, and errors.
- [ ] 4.7 Add view-model tests for command submission, sequence increments, status updates while input is preserved, and server message presentation.

## 5. Verification

- [ ] 5.1 Run `cargo fmt` for the workspace.
- [ ] 5.2 Run `cargo test` for the workspace.
- [ ] 5.3 Run `cargo check` for the workspace.
- [ ] 5.4 Run `openspec validate --all`.
- [ ] 5.5 Manually verify the local demo by starting `space-server`, connecting with `space-client-tui`, and running `objects`, `distance mars`, `distances --limit 10`, `distances --sort distance`, `status`, and `quit`.
