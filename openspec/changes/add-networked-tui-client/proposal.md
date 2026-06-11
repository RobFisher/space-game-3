## Why

The project has a reusable ephemeris library but no end-to-end slice that proves a player-facing client can query game data through an authoritative server boundary. A minimal networked TUI demo will validate the first client/server protocol, server-owned ephemeris access, and Ratatui application loop without committing to the final simulation or multiplayer architecture.

## What Changes

- Add a shared `space-game-protocol` crate for JSON-serializable client/server messages and view DTOs.
- Add a `space-server` crate that exposes a local WebSocket endpoint, owns ephemeris queries, and serves object and distance requests.
- Add a demo fictional solar-system registry using supported ephemeris sources such as static state, circular orbit, and fixed offset objects.
- Add a `space-client-tui` crate that connects to the server, renders an output log, status pane, and command input, and sends user commands over the protocol.
- Support first-slice commands for help, object listing, single-object distance, multiple distances, status, and clean exit.
- Keep persistence, authentication, combat, trading, navigation, and full simulation scheduling out of scope.

## Capabilities

### New Capabilities

- `network-protocol`: Shared wire-visible messages and DTOs for the first JSON WebSocket protocol.
- `authoritative-space-server`: A local authoritative server that owns ephemeris access and answers object, distance, and status queries.
- `tui-space-client`: A Ratatui client that connects to the server, displays status/output, accepts commands, and exits cleanly.

### Modified Capabilities

- None.

## Impact

- Workspace membership expands beyond the existing `space-game` and `space-game-ephemeris` crates.
- New dependencies are expected for WebSocket serving/client transport, async runtime, JSON serialization, terminal UI, terminal events, logging, and error handling.
- The server depends on `space-game-ephemeris` and `space-game-protocol`; the TUI client depends on `space-game-protocol` but not directly on `space-game-ephemeris`.
- The first demo data set is fictional and data-driven so it remains compatible with the currently implemented ephemeris source variants.
