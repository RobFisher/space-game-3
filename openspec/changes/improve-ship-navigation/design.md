## Context

The current server owns the player ship, simulation time, ephemeris queries, and flight plan creation. Flight planning is split between `ship.rs` and `query.rs`: the ship owns plan storage and interpolation, while the query service estimates moving-object intercepts and resolves the target object. Existing plans travel to a snapshotted target object state and then hand off to a default fictional orbit.

The new navigation work should preserve the server-authoritative model and current client command flow while creating a dedicated place for navigation math. The immediate gameplay model remains approximate: use bounded acceleration, configurable circular arrival orbits, and a short orbit-entry phase to hide position/velocity discontinuities. Full fuel accounting and precise orbital mechanics are deferred.

## Goals / Non-Goals

**Goals:**

- Introduce a `navigation` module that owns flight profile planning, target prediction, arrival orbit selection, orbital estimates, and phase-based state resolution.
- Treat acceleration in G as a user-facing input/display format while storing normalized kilometers-per-second-squared values in plan data.
- Plan flights to circular arrival orbit insertion states instead of object centers.
- Support default, low, stationary/geosynchronous, and custom circular orbit requests where required body data is available.
- Calculate and expose orbital radius, altitude, period, and circular speed when the destination body has enough navigation constants.
- Add an approximate `entering_orbit` phase between powered transfer and exact circular orbiting motion.
- Keep specific impulse available as ship/navigation profile metadata for later fuel calculations.

**Non-Goals:**

- Exact n-body navigation, patched conics, Lambert solvers, collision handling, or continuous guidance.
- Fuel consumption, mass changes, propellant accounting, or engine selection.
- Multiple queued plans, persisted flight history, or multi-ship navigation.
- Elliptical or inclined custom orbit planning.

## Decisions

### Add a dedicated server navigation module

Create a server-owned navigation module that exposes planning and state-resolution functions. `PlayerShip` remains the owner of identity, name, current motion, active plan, and plan numbering. `SolarSystemQueryService` remains the orchestrator that resolves objects and authoritative time, but delegates route calculation to navigation.

Alternative considered: keep extending `ship.rs`. That would be faster for the next few fields, but it would keep mixing identity/state ownership with planning algorithms and make later fuel/orbit improvements harder to isolate.

### Store normalized units, display G

Flight plans store `acceleration_km_s2`, plus optionally the user-facing G value used to create the plan. Command parsing accepts both existing numeric km/s² values and G-suffixed values such as `0.5g`. DTO/UI output includes acceleration in G for readability.

Alternative considered: store G everywhere. That is convenient for UI, but simulation and ephemeris code already use kilometers and seconds.

### Model the destination as an arrival orbit

Flight plan targets continue to reference a destination object, but the concrete terminal state becomes an arrival orbit insertion state around that object. The plan records an arrival orbit request and a resolved circular orbit estimate. The transfer endpoint is a point on that orbit near the estimated arrival time, not the object's center.

Alternative considered: retain target-center transfer and only change post-arrival orbiting. That preserves the existing behavior but does not solve the visible arrival snap.

### Keep orbit presets data-driven

Support preset names in the navigation model:

- `default`: current default fictional ship orbit radius behavior.
- `low`: body-specific low orbit altitude when configured, otherwise a generic low-orbit fallback.
- `stationary`: a circular orbit whose period matches body rotation when gravitational and rotation constants are known.
- `custom`: an explicit radius or altitude.

Stationary/geosynchronous requests fail clearly when required constants are unavailable rather than silently producing a misleading orbit. Orbital period and circular speed are calculated only when a gravitational parameter is known.

Alternative considered: hard-code Earth-only presets. That would support the initial demo but would not fit the existing multi-object registry direction.

### Use a phase-based approximate plan

The first implementation keeps deterministic approximate transfer math but records explicit phases:

1. `powered_transfer`: accelerate/decelerate profile from origin to the chosen insertion state.
2. `entering_orbit`: short smoothing phase from the transfer endpoint toward the exact circular orbit state.
3. `orbiting`: normal circular orbit motion after orbit entry completes.

The `entering_orbit` phase blends position and velocity toward the circular orbit solution over a fixed or configured duration. This is intentionally approximate and exists to preserve gameplay continuity until richer navigation exists.

Alternative considered: solve for exact terminal velocity now. That is more correct, but it would expand scope into orbital transfer math before the game has fuel, mass, or maneuver planning.

### Keep specific impulse non-authoritative for now

Add specific impulse to the navigation/engine profile as metadata, validate it when configured, and expose it where useful. Do not use it to compute fuel until the ship has mass and propellant state.

Alternative considered: omit specific impulse. That would be simpler, but including it now helps shape the module boundary around engine capabilities without committing to fuel accounting.

## Risks / Trade-offs

- Approximate transfer may not match physically accurate terminal velocity -> Use the explicit `entering_orbit` phase and label it separately.
- Stationary orbit requires body rotation and gravitational constants that may not exist for every object -> Return a clear planning error and allow default/low/custom alternatives.
- Adding DTO fields can affect clients -> Use serde defaults for newly optional fields where practical and update protocol tests.
- G input can be ambiguous for bare numbers -> Preserve current bare-number km/s² behavior and require `g` suffix for G input.
- Navigation constants may start as a small server-side table -> Keep the model isolated so constants can later move into ephemeris metadata.

## Migration Plan

1. Add navigation types and pure unit/orbit calculation tests.
2. Move current transfer duration/interpolation behavior behind navigation APIs without changing commands.
3. Add arrival orbit request/estimate data and update plan creation.
4. Add command parsing, protocol fields, and TUI display for G/orbit options.
5. Add `entering_orbit` state resolution and query behavior.
6. Run targeted Rust tests and `openspec validate --all`.
