## Why

The server can calculate distances, but it still treats the observer as a loose label, frame, and vector rather than as a first-class spatial state. Promoting positions to state vectors at the server boundary gives distance queries, future ships, movement, and location readouts a shared model without starting full navigation or flight planning yet.

## What Changes

- Represent the server-owned observer internally as a spatial state with frame, epoch, position, velocity, and quality.
- Refactor distance and distances queries so they derive distance from target and observer state vectors in a compatible frame.
- Add a `where` command that reports a simple authoritative location summary for the current observer at the effective simulation time.
- Add protocol DTOs for location summaries without exposing raw x/y/z coordinates by default.
- Update the TUI and plain text mode to display the `where` result as a readable landmark-based summary.
- Keep raw coordinates, player ships, navigation, flight planning, persistence, and real frame transforms out of scope for this change.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `authoritative-space-server`: Server distance handling changes from raw fixed-observer vector math to spatial-state-derived distance math, and command handling gains `where`.
- `network-protocol`: Protocol messages gain a location summary response that can carry the `where` command result.
- `tui-space-client`: The TUI and plain text client present the new location summary without relying on raw coordinates by default.

## Impact

- Affected crates: `crates/space-server`, `crates/space-game-protocol`, and `crates/space-client-tui`.
- Existing distance commands and protocol distance responses remain backward compatible.
- No new runtime dependencies are expected.
- The ephemeris crate's existing `StateVector` and `FrameId` model should be reused rather than duplicated.
