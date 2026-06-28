## Why

Ship motion currently mixes flight planning, transfer interpolation, and orbit handoff inside the server ship/query code, and transfers arrive at a target object rather than a planned orbital insertion state. Moving navigation into a dedicated capability lets flight plans use clearer Newtonian-style engine parameters, configurable arrival orbits, and smoother transitions into orbit.

## What Changes

- Add a dedicated ship navigation module responsible for transfer planning, target prediction, orbit selection, orbital estimates, and phase-based state resolution.
- Change flight planning from target-center arrival to configurable circular arrival orbits with presets such as default, low orbit, and stationary/geosynchronous orbit when body data supports it.
- Accept acceleration in user-friendly G units while preserving normalized simulation units internally.
- Add approximate velocity continuity at arrival by introducing an `entering_orbit` phase before the ship fully transitions to orbiting motion.
- Extend flight plan data and presentation with arrival orbit details such as radius, altitude, orbital period, and circular speed when they can be calculated.
- Keep specific impulse as an engine/navigation profile field for future fuel calculations without requiring fuel accounting in this change.

## Capabilities

### New Capabilities

- `ship-navigation`: Dedicated navigation planning and resolution for player ship transfers, engine limits, arrival orbit selection, orbital estimates, and orbit-entry phases.

### Modified Capabilities

- `ship-flight-plans`: Flight plans target orbital insertion states, include navigation/orbit metadata, support G acceleration input, and resolve phase-based transfer/orbit-entry motion.
- `player-ship-state`: Ship state resolution can report `entering_orbit` motion and hand off smoothly from transfer motion to circular orbiting motion.
- `network-protocol`: Flight plan and ship state protocol messages expose navigation phase and arrival orbit estimate fields.
- `authoritative-space-server`: Server command parsing and query handling support orbit options, G acceleration input, and navigation-module planning.
- `tui-space-client`: The client displays navigation phase, acceleration in G, orbit selection, orbital period, and arrival orbit estimates.

## Impact

- Affected Rust modules: `crates/space-server/src/ship.rs`, `crates/space-server/src/query.rs`, command handling, and a new server navigation module.
- Affected protocol crate: flight plan DTOs, ship motion labels, and serialization tests.
- Affected TUI: command completion/help text, plain and interactive flight plan display, and status-pane flight information.
- Affected specs: new `ship-navigation` spec plus deltas for existing flight plan, ship state, network protocol, authoritative server, and TUI capabilities.
- No external runtime dependency is expected.
