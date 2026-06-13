# Space game planned design overview

This document summarises the current planned design direction for the Rust TUI real-time space adventure game. It is intended to ground OpenSpec exploration, change proposals and implementation work.

The project currently has:

- A Rust TUI client.
- A server/backend that the TUI can query.
- A `space-game-ephemeris` crate that models solar system bodies.
- Some simple custom objects in orbit.
- TUI commands that can query objects and distances.

The next design work should move the project from “querying distances to objects” towards a coherent simulation model with time, spatial state, player ships, navigation and eventually persistence/multiplayer.

---

## High-level architecture

The preferred architecture keeps the TUI client separate from the simulation/game server.

The TUI should be a client UI, not the source of truth for simulation state. It should send commands and queries to the server and render the returned state.

The server should own:

- Simulation time.
- Ephemeris access.
- Game object state.
- Player ship state.
- Future flight plans.
- Future persistence.
- Future multiplayer coordination.

This separation matters because the client and server may later run on different computers, and there may eventually be multiple clients connected to the same game.

---

## Current position in development

The game has reached an early integration milestone:

- The TUI can talk to the server.
- The server can use the ephemeris layer.
- Solar system objects and simple custom orbital objects can be queried.
- Distances can be calculated and shown in the TUI.

The next changes should be small, OpenSpec-sized increments that establish core concepts cleanly.

Suggested order:

1. Add simulation time.
2. Introduce a spatial state model: position plus velocity.
3. Add player ship state.
4. Add basic flight plans.
5. Add an ephemeris manifest/downloader.
6. Improve TUI navigation and position display.
7. Later: persistence, multiplayer, proper delta-v/navigation modelling.

---

## Simulation time

The server should have a first-class notion of simulation time.

Initial requirements:

- The server maintains a current simulation timestamp.
- The TUI can query the current simulation time.
- The TUI can advance simulation time manually, for example by seconds, minutes, hours or days.
- Existing distance queries should use the current simulation time by default.
- Queries may optionally accept an explicit timestamp for deterministic testing and debugging.
- No real-time ticking loop is required at first.

Design preference:

- Keep this deterministic and testable.
- Avoid tying simulation time directly to wall-clock time in the first version.
- Make it easy to add pause/resume/realtime ticking later.

Example future commands:

```text
/time
/time advance 1 day
/time advance 6 hours
/distance mars
/distance mars at 2027-03-27T12:00:00Z
```

---

## Spatial state model

The internal model should move beyond “position” and “distance”.

The preferred abstraction is a spatial state:

```rust
SpatialState {
    frame,
    epoch,
    position_km: Vec3,
    velocity_km_s: Vec3,
}
```

A state should include:

- A reference frame.
- An epoch/timestamp.
- A 3D position vector, probably in kilometres.
- A 3D velocity vector, probably in kilometres per second.

This matches the shape of SPICE/JPL/ANISE-style Cartesian state vectors and gives the project room to support relative velocity, navigation and delta-v later.

Distances should be derived from two spatial states:

```text
distance = |target.position - observer.position|
```

Relative velocity should also be derived from two spatial states:

```text
relative_velocity = target.velocity - observer.velocity
relative_speed = |relative_velocity|
```

This allows future readouts such as:

```text
Nearest body: Earth
Distance: 42,000 km
Relative speed: 3.2 km/s
```

---

## Coordinate frames

The project should be explicit about coordinate frames from early on.

Initial frame support can be deliberately small. A likely starting point is:

- Solar system barycentric frame for ephemeris-derived body states.
- Optional heliocentric or body-relative display frames later.

The internal server model should not rely on user-facing descriptions such as “near Mars” as the source of truth. It should resolve objects into spatial states at a given epoch.

Important rule:

- Raw coordinates are useful internally and for debug output.
- Raw coordinates should not be the main way the TUI explains location to the player.

The TUI can describe position using:

- Nearest major body.
- Distance to selected landmarks.
- Relative speed to nearest body.
- Current frame of reference.
- Current simulation time.
- Active flight plan summary.

---

## Ephemeris layer

The `space-game-ephemeris` crate should provide a clean interface for resolving solar system body states.

The underlying ephemeris approach is expected to support state vectors: position and velocity at an epoch in a reference frame.

The crate should expose, or be refactored to expose, full spatial states rather than only positions or distances.

Preferred interface shape:

```rust
fn state_of(body_id: BodyId, epoch: Epoch, frame: FrameId) -> Result<SpatialState>;
```

or similar.

Distance-related methods can remain as convenience helpers, but they should be derived from state vectors.

Potential convenience helpers:

```rust
fn distance_between(a: ObjectId, b: ObjectId, epoch: Epoch) -> Result<Distance>;
fn relative_state(a: ObjectId, b: ObjectId, epoch: Epoch) -> Result<RelativeState>;
fn nearest_body(position: SpatialState, epoch: Epoch) -> Result<NearestBodyInfo>;
```

---

## Custom orbital objects

The game also has simple custom objects in orbit, such as development/test objects or future stations.

These should use the same state interface as ephemeris-backed solar system bodies.

That means a custom orbital object should be able to resolve to:

- Position at an epoch.
- Velocity at an epoch.
- Reference frame.

This prevents the rest of the server from needing to know whether an object is:

- A real solar system body from ephemeris data.
- A simple artificial satellite/station.
- A future gameplay object with custom motion.

Design preference:

```rust
trait StateProvider {
    fn state_at(&self, epoch: Epoch, frame: FrameId) -> Result<SpatialState>;
}
```

The exact trait name and structure can vary, but the design should aim for polymorphism over object types.

---

## Player ship state

The next gameplay object should be the player ship.

Initial player ship state should include:

- Ship id.
- Display name.
- Current spatial state or a way to resolve its state at the current simulation time.
- Optional current flight plan.

At first, the game can support one player ship without full authentication or multiplayer identity.

The TUI should be able to query:

- “Where is my ship?”
- “What is my ship nearest to?”
- “How far is my ship from Mars?”
- “What is my speed relative to Earth?”

Example future commands:

```text
/ship
/ship where
/ship distance mars
/ship relative earth
```

Do not build full multiplayer or persistence as part of the first ship-state change.

---

## Flight plans

Flight plans should come after simulation time, spatial state and ship state.

The first version should be deliberately simple and deterministic. It does not need physically accurate orbital transfer maths.

Initial flight plan fields:

- Flight plan id.
- Ship id.
- Origin spatial state or origin object.
- Target object id or target spatial state.
- Departure time.
- Arrival time or duration.
- Status: planned, active, completed, cancelled.

The first implementation can use simple interpolation between origin and target states, while leaving room for better navigation later.

Important design question:

- If the target is a moving object, should the target position be snapshotted at arrival time, continuously tracked, or resolved when the plan is created?

Suggested first answer:

- Resolve the origin at departure time.
- Resolve the target at planned arrival time.
- Interpolate between those two spatial states.
- Clearly document that this is an abstract travel model, not realistic orbital mechanics.

This supports gameplay quickly while preserving the option to add better manoeuvre planning later.

Example future commands:

```text
/flight plan mars 30 days
/flight active
/flight cancel
```

---

## Delta-v and relative velocity

Delta-v should not be the first navigation feature, but the model should be designed so it can be added later.

Relative velocity is the foundation:

```text
ship_velocity_relative_to_body = ship.velocity - body.velocity
relative_speed = |ship_velocity_relative_to_body|
```

Delta-v is not the same as current relative speed. Delta-v is the required change in velocity to achieve a manoeuvre, such as:

- Match velocity with a target.
- Enter orbit.
- Escape a body.
- Transfer between bodies.
- Brake at arrival.

Suggested modelling layers:

| Layer | Purpose |
|---|---|
| Position only | Distances to objects |
| Position + velocity | Relative speed and motion context |
| Abstract flight plan | Game-friendly travel |
| Patched conics / manoeuvres | Approximate delta-v gameplay |
| Numerical integration | More realistic gravity/thrust simulation |

For now, implement position and velocity cleanly, then expose relative speed in the TUI. Full delta-v planning can come later.

---

## Ephemeris manifest and downloader

The project should eventually manage real ephemeris files through a manifest-driven system.

This is important for open source development because large data files may not belong directly in the repository.

Initial manifest asset fields:

- Name.
- Source URL.
- Local path.
- Expected size, if useful.
- Checksum, if available.
- Licence/source notes.

Design preference:

- Do not download files implicitly during normal library compilation.
- Provide an explicit dev command, CLI helper or build helper to fetch missing files.
- Runtime errors for missing required files should be clear and actionable.
- Tests should not require internet access.
- Unit tests should use small fixtures or mocked providers.

Possible commands:

```text
space-game-ephemeris fetch-assets
space-game-ephemeris verify-assets
```

The exact command shape can vary.

---

## TUI design direction

The TUI should make space understandable without requiring the player to read raw coordinates.

Useful player-facing location output:

```text
Simulation time: 2027-03-27 12:00 UTC
Ship: Wayfarer
Nearest body: Earth
Distance to Earth: 42,000 km
Relative speed to Earth: 3.2 km/s
Distance to Mars: 88.4 million km
Active flight plan: none
```

Potential TUI areas:

- Output pane: command results, discoveries, messages, logs.
- Status pane: current time, ship location summary, nearest body, active flight plan.
- Command entry: command input with autocomplete.

Potential commands:

```text
/time
/time advance 1 day
/objects
/object mars
/distance mars
/ship
/ship where
/ship relative earth
/flight plan mars 30 days
/flight active
```

The TUI may expose raw coordinates behind a verbose or debug flag:

```text
/ship where --debug
/object mars --raw
```

---

## Server API/message design

The exact transport can remain flexible, but the server/client boundary should be clear.

The TUI should send requests such as:

- Get current simulation time.
- Advance simulation time.
- List known objects.
- Get object summary.
- Get object spatial state.
- Get distance between two objects.
- Get relative state between two objects.
- Get player ship state.
- Create/cancel/query flight plan.

The server should return structured data, not preformatted terminal text, where possible. The TUI should own presentation formatting.

For early development, it is acceptable to keep messages simple. But avoid baking in assumptions that only one in-process client will ever exist.

---

## Persistence direction

Persistence is not part of the immediate next change, but the design should avoid making it hard.

Eventually the server should be able to resume after a stop or crash from recently saved state.

Likely persistent state:

- Simulation time.
- Player profiles.
- Ship state.
- Flight plans.
- Custom game objects.
- Message/event log.
- World state snapshots.

Ephemeris-backed solar system bodies do not need to be persisted as state because their positions are derived from ephemeris data and time.

Custom objects and ships do need persistence.

The game is not expected to be fast-paced and may only have a few dozen players at first, but scalability should be kept in mind.

DynamoDB is a possible storage backend because it is cheap, fast and scalable, but the project should not prematurely bake DynamoDB assumptions into the core simulation types.

---

## Multiplayer direction

Multiplayer is a future goal.

The current design should keep multiplayer possible by ensuring:

- The server owns authoritative game state.
- The TUI is a client.
- Player/ship identity can be added later.
- Commands are structured and can be validated server-side.
- State changes can eventually be persisted and broadcast.

Do not implement multiplayer in the next small changes unless explicitly scoped.

---

## Testing guidance

Tests should focus on deterministic simulation behaviour.

Useful test areas:

- Simulation time starts at a known value.
- Advancing time by a duration changes the timestamp correctly.
- Object state queries are deterministic for a fixed epoch.
- Distance is derived correctly from two positions.
- Relative velocity is derived correctly from two velocities.
- Custom orbital objects expose both position and velocity.
- Ship state can be resolved at simulation time.
- Flight plan interpolation is deterministic.
- Advancing time past arrival updates or resolves flight status correctly.
- Missing ephemeris assets produce clear errors.
- Asset downloader tests do not require internet access.

Prefer unit tests around core model types and integration tests around server/client commands.

---

## Near-term OpenSpec-sized changes

### 1. Simulation time

Add server-owned simulation time and TUI commands to query and advance it.

Avoid realtime ticking at first.

### 2. Spatial state model

Introduce `SpatialState` with frame, epoch, position and velocity.

Refactor distance queries to derive from states.

Expose relative velocity/speed as a derived value.

### 3. Player ship state

Add a simple player ship entity and TUI commands to query its location and distances.

No multiplayer identity yet.

### 4. Basic flight plans

Allow registering a simple abstract flight plan to a target object over a duration.

Use deterministic interpolation first.

No realistic orbital transfer maths yet.

### 5. Ephemeris manifest/downloader

Add manifest-driven asset management for real ephemeris files.

Avoid network access in tests and normal compilation.

### 6. TUI navigation display

Improve `where am I?`, status pane and object summary output.

Show nearest body, distance, relative speed and active flight plan.

Avoid raw coordinates by default.

---

## Design principles

Use these principles when exploring or implementing changes:

1. Keep each OpenSpec change small.
2. Prefer deterministic simulation over real-time behaviour at first.
3. Keep the TUI separate from the authoritative server state.
4. Model state vectors, not just distances.
5. Make distance and relative velocity derived values.
6. Be explicit about units and reference frames.
7. Keep user-facing location output friendly and game-like.
8. Avoid full physics until the simpler gameplay model exists.
9. Do not make tests depend on network access or large ephemeris files.
10. Keep future multiplayer and persistence possible, but do not implement them too early.
