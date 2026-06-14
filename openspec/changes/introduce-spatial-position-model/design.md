## Context

The ephemeris crate already exposes the right primitive for this change: `StateVector` contains position, velocity, frame, epoch, and quality. Solar system bodies and authored orbital objects can resolve through `SolarSystem::state`, while the server currently keeps its observer as `ObserverLocation { label, frame, position_km }` and calculates distances by subtracting the target position from that raw vector.

The next gameplay features need a stronger server-side model for "where something is" without requiring the TUI to expose Cartesian coordinates to the player. This change promotes the server observer to a state-vector-like model, keeps distance responses backward compatible, and adds a `where` command that reports a landmark-based location summary for the observer or a named object.

## Goals / Non-Goals

**Goals:**

- Use `StateVector` as the internal position model at the server query boundary.
- Represent the current observer as a spatial state at the effective simulation time.
- Derive `distance` and `distances` from observer and target states rather than from a raw observer vector.
- Add a `where` command and protocol response that summarize the current observer location or a named object location.
- Keep TUI output simple and landmark-based, with no default raw coordinate display.

**Non-Goals:**

- Add player ship state.
- Add navigation, movement, flight planning, or orbital transfer behavior.
- Implement heliocentric transforms, body-fixed transforms, or arbitrary frame conversion.
- Expose raw x/y/z coordinates by default in the TUI.
- Replace the ephemeris crate's existing `StateVector` type with a parallel model.

## Decisions

### Reuse `StateVector` as the internal spatial model

The server should reuse `space_game_ephemeris::StateVector` for authoritative position data instead of introducing a second coordinate type. This preserves one representation for frame, epoch, position, velocity, and quality across solar system bodies, custom orbital objects, and future ship state.

Alternative considered: introduce a new `SpatialPosition` type containing only frame, epoch, and position. That would match the smallest wording of this change, but it would immediately need velocity again for relative-speed and movement features already planned in the design overview.

### Keep barycentric state as the canonical first-frame target

Distance math in this change should only operate on states that are already in a compatible frame, with `SolarSystemBarycentricJ2000` as the expected canonical global frame. Parent-centered local frames remain useful inside ephemeris source resolution, but server distance queries should consume resolved global states.

Alternative considered: add heliocentric and body-relative display frames now. That expands the frame transform surface before the game needs it and risks confusing display concerns with authoritative state.

### Model the observer as a state provider

The fixed demo observer should become a value that can produce a `StateVector` at a requested epoch. Initially this can be a static state with zero velocity and fictional quality. The important boundary is that query code asks for observer state at an epoch rather than reading a naked position vector.

Alternative considered: register the observer as another ephemeris object. That may be useful later, but it would mix a server viewpoint into the demo object registry and make the current change larger than necessary.

### Add a location summary DTO instead of coordinate DTOs

The protocol should add a subject-oriented location summary response for `where` containing subject identity/label, frame, simulation time, nearest known object, distance in kilometers and AU, and optional quality. The DTO should not include raw coordinates by default.

Alternative considered: return the full state vector through the protocol. That would be useful for debugging but would make raw coordinates part of the first user-facing contract.

### Use nearest known object as the initial location language

The first `where` output should describe the observer or named object relative to the nearest other known object at the effective simulation time. This gives the TUI a player-readable answer while leaving richer descriptions such as "in orbit around Earth" or "near Mars" for later.

Alternative considered: only report observer label and frame. That would prove the state model exists but would not improve user-facing location beyond current status output.

### Preserve bare `where` as the observer shortcut

Bare `where` should continue to summarize the current observer at the current simulation time. `where <object>` should summarize a named object at the current simulation time, and `where <object> --at <timestamp>` should summarize that object at an explicit time. This mirrors the `distance <object> [--at <timestamp>]` command shape without adding coordinate display or arbitrary frame transforms.

## Risks / Trade-offs

- Frame mismatch errors could become visible once distance code checks state compatibility. Mitigation: keep all first-slice server states in `SolarSystemBarycentricJ2000` and return clear query errors for incompatible frames.
- The nearest-object summary can be simplistic for large-scale orbital contexts. Mitigation: specify it as an initial Euclidean nearest-known-object summary, not as a navigation or orbital classification.
- Adding protocol messages expands client/server API surface. Mitigation: add a new response variant while preserving existing distance messages and command behavior.
- Treating the observer as static means bare `where` is not yet a ship location. Mitigation: name the output as the current observer/location summary and keep player ship state explicitly out of scope.

## Migration Plan

This is an in-repo protocol expansion with no data migration. Existing clients can continue to use `objects`, `distance`, `distances`, `status`, `time`, and `advance`; the new `where` command and location response are additive. Rollback is a code revert to the previous observer-location and distance-query implementation.

## Open Questions

- Should a later debug command expose raw state vectors through `where --debug`, or should that live behind a separate diagnostic command?
- When player ship state is introduced, should bare `where` switch to the player ship, or should the command distinguish `where observer` and `ship where`?
