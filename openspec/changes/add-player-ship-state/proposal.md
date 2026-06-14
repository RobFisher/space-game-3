## Why

The current game slice measures distance and location from a fixed demo observer, but the planned gameplay model needs the player to inhabit a named ship. Introducing a single player-controlled ship now gives the server a concrete gameplay subject for status, location, and distance queries before multiplayer, persistence, flight planning, or thrust mechanics are added.

## What Changes

- Add a server-owned player ship state with an id, player-editable display name, motion mode, and state resolution at a requested simulation time.
- Place the default player ship in a fictional circular orbit near Earth using the existing authoritative simulation clock.
- Treat the player ship as the subject for `status`, `where`, `distance <object>`, and `distances` queries that previously used the fixed observer.
- Add command handling for querying ship status and renaming the player ship.
- Extend protocol status and ship response DTOs so clients can display the current ship name, motion mode, frame, and resolved simulation time.
- Update the TUI and plain text client presentation so the status pane and status output show the player ship name.
- Do not implement multiplayer identity, persistence, flight planning, or thrust simulation in this change.

## Capabilities

### New Capabilities

- `player-ship-state`: Server-owned single-player ship identity, name, motion mode, and spatial state resolution.

### Modified Capabilities

- `authoritative-space-server`: Replace fixed observer-centered query behavior with player-ship-centered status, distance, and location behavior; add ship query and rename commands.
- `network-protocol`: Add wire-visible ship status/state fields and messages while keeping protocol DTOs independent of server and ephemeris crates.
- `tui-space-client`: Present ship status/name in the status pane and plain mode output, and send/display ship query and rename command results.

## Impact

- Affected crates: `space-server`, `space-game-protocol`, and `space-client-tui`.
- Affected command behavior: `status`, `where`, `distance`, `distances`, plus new ship commands for status/name changes.
- Affected tests: protocol serialization tests, server query/command tests, WebSocket integration tests, TUI view model/rendering tests, and plain text output tests.
- No new runtime dependencies are expected.
