## Why

Player ships can currently be queried as spatial state, but they cannot be given a destination or move under player intent. A small flight plan model gives the first useful navigation loop: choose a known object, choose an acceleration, and let server-owned ship state move through simulation time.

## What Changes

- Add a server-owned flight plan model for the single player ship, including id, origin state, target object intercept estimate, departure time, calculated arrival time/duration, acceleration, and status.
- Resolve player ship position from an active flight plan when one exists, using deterministic accelerate-then-decelerate interpolation between the origin and a snapshotted predicted arrival target.
- Estimate moving-object arrivals by iteratively resolving the target object at predicted arrival times, then snapshot the final target position for deterministic in-flight behavior.
- Allow users to create a new flight plan from the ship's current resolved position at the current authoritative simulation time, replacing any active plan with the new acceleration.
- Allow users to query the current flight plan and cancel it.
- Update protocol and TUI presentation for flight plan responses.

## Capabilities

### New Capabilities
- `ship-flight-plans`: Defines player ship flight plan lifecycle, target estimation, interpolation, replacement, cancellation, and arrival behavior.

### Modified Capabilities
- `player-ship-state`: Player ship state resolution can come from active flight plan motion in addition to orbiting motion.
- `authoritative-space-server`: Server command handling gains flight plan creation, status, and cancellation commands.
- `network-protocol`: Shared protocol gains serializable flight plan response DTOs.
- `tui-space-client`: TUI can submit flight plan commands and display flight plan responses.

## Impact

- Affects `space-server` ship state, query service, command parsing, and tests.
- Affects `space-game-protocol` DTOs and serialization tests.
- Affects `space-client-tui` message display and plain mode formatting.
- No external service or new runtime dependency is expected.
