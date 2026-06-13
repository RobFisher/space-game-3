## Why

The server currently reports and uses a fixed default game time for status and distance queries, so orbital results do not progress during a session. A first-class simulation clock lets the server remain authoritative for time while the TUI can show an advancing clock and clients can query or advance time deterministically.

## What Changes

- Add a server-owned simulation clock initialized from the configured default game time.
- Make the clock advance with elapsed wall time at a deterministic rate, with no per-second mutation loop required.
- Allow clients to query the current simulation time.
- Allow clients to manually advance simulation time by seconds, minutes, hours, or days.
- Make distance queries use the current server simulation time unless the request explicitly supplies a timestamp.
- Let the TUI display an updating simulation clock by projecting the latest server clock sample between refreshes.

## Capabilities

### New Capabilities
- `simulation-clock`: Server-owned simulation time, clock querying, manual advancement, and deterministic clock behavior.

### Modified Capabilities
- `authoritative-space-server`: Distance and status queries use the server-owned simulation clock instead of a fixed default timestamp.
- `network-protocol`: Protocol messages represent simulation time queries, time advancement, and optional explicit timestamps for distance requests.
- `tui-space-client`: The TUI can fetch simulation time, display an advancing clock, and submit time advancement commands.

## Impact

- Affects `space-server` application state, command handling, WebSocket handling, and tests.
- Affects shared protocol DTOs in `space-game-protocol`.
- Affects TUI state, rendering, command formatting, plain-mode output, and tests in `space-client-tui`.
- Reuses the existing ephemeris `GameTime` type; no new runtime dependency is expected.
