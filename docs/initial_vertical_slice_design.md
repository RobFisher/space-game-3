# Design brief: minimal networked TUI client for solar-system distance queries

## Purpose

Create a minimal vertical slice for the Rust space game that ties together:

- the existing `crates/space-game-ephemeris` solar-system model,
- a lightweight authoritative `space-server`,
- a shared client/server protocol,
- and a rudimentary Ratatui TUI client.

The immediate gameplay/demo goal is simple: a player can connect with the TUI client, list known solar-system objects, and request their distances from a fixed observer location at the current game timestamp.

This is not intended to be the final game UI or full simulation architecture. It is a practical first integration slice that proves the ephemeris model, server boundary, protocol messages and Ratatui client can all work together.

---

## Explore-mode instruction

Please inspect the existing repository first, especially `crates/space-game-ephemeris`, before proposing changes.

The desired output from explore mode is:

1. A proposed minimal crate/module layout.
2. A protocol design for the first few client/server messages.
3. A server design that uses `space-game-ephemeris` to answer object/distance queries.
4. A Ratatui client design with a basic OpenCode-style layout.
5. A small implementation plan, broken into safe steps.
6. Notes on risks, unknowns and interfaces that need confirming from the current code.

Do not over-design multiplayer, persistence, authentication, combat, trading, navigation or full game simulation yet. Leave clear extension points, but focus on the first working vertical slice.

---

## Current known context

The workspace currently includes:

```text
crates/space-game-ephemeris
```

This crate is expected to provide, or be extended to provide, a model of solar-system objects and their positions at a given timestamp.

The future architecture is expected to separate:

```text
Ratatui TUI client
  <-> shared protocol / transport
  <-> space-server
  <-> space-game-ephemeris / game engine
```

The server should be authoritative. The client should not call `space-game-ephemeris` directly except perhaps in tests or temporary debug utilities.

---

## User-facing behaviour for the first slice

The TUI client should let the user connect to a local server and run commands such as:

```text
help
objects
distance mars
distances
distances --limit 10
distances --sort distance
status
quit
```

Exact command names can change during design, but the first build should support at least:

- listing available solar-system objects,
- querying the distance to one named object,
- querying a sorted list of distances to multiple known objects,
- showing current connection/game-time status,
- and exiting cleanly.

The UI should be intentionally basic but structured.

Suggested layout:

```text
+-------------------------------------------------------+----------------------+
| Output / event log                                    | Status               |
|                                                       |                      |
| > objects                                             | Connected: yes       |
| Known objects: Sun, Mercury, Venus, Earth...          | Server: localhost    |
|                                                       | Game time: ...       |
| > distance mars                                       | Observer: demo-origin|
| Mars: 1.23 AU / 184,000,000 km                        | Objects: 17          |
|                                                       |                      |
+-------------------------------------------------------+----------------------+
| Command: distances --limit 10                                                |
+------------------------------------------------------------------------------+
```

The status pane should be able to update independently of the user's command entry. For example, it can show connection status, current game timestamp, observer location name and last server update time.

---

## Fixed observer location

For this first slice, distances should be measured from a fixed observer location owned by the server.

Recommended design:

```rust
pub enum ObserverLocation {
    FixedCartesian {
        frame: CoordinateFrame,
        x_km: f64,
        y_km: f64,
        z_km: f64,
        label: String,
    },
}
```

The implementation should use the coordinate frame already used by `space-game-ephemeris` if one exists.

For the MVP, use a clearly labelled hard-coded location, for example:

```text
label: "demo-origin"
position: [0.0, 0.0, 0.0]
frame: ephemeris-native-frame
```

If the current ephemeris API does not make the native coordinate frame clear, the explore output should call this out explicitly and recommend the smallest clarification needed.

Do not introduce custom stations or moving observer objects yet unless that is already easy in the existing ephemeris design. The point is to prove the query path, not to design the final navigation model.

---

## Recommended workspace shape

The explore should check what already exists and then recommend the smallest sensible structure. A likely target is:

```text
crates/
  space-game-ephemeris/   Existing solar-system model
  space-game-protocol/    Shared serialisable protocol types
  space-server/           Authoritative server using ephemeris
  space-client-tui/       Ratatui client
```

If the repository already has a different naming/layout convention, follow that convention instead.

### `space-game-protocol`

Owns wire-visible types only:

- client-to-server messages,
- server-to-client messages,
- DTO/view types,
- object IDs/names used in protocol,
- distance result types,
- error types suitable for display.

It should not depend on Ratatui, Crossterm, Axum or the full server implementation.

Likely dependencies:

```toml
serde = { version = "1", features = ["derive"] }
thiserror = "1"
uuid = { version = "1", features = ["serde", "v4"] }
time = { version = "0.3", features = ["serde"] }
```

Use versions consistent with the repository if they already differ.

### `space-server`

Owns:

- server startup,
- WebSocket endpoint,
- connected client sessions,
- command handling,
- ephemeris queries,
- fixed observer location,
- server-side game timestamp.

Likely dependencies:

```toml
tokio = { version = "1", features = ["full"] }
axum = { version = "0.7", features = ["ws"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
space-game-ephemeris = { path = "../space-game-ephemeris" }
space-game-protocol = { path = "../space-game-protocol" }
```

If the workspace is already using newer Axum/Tokio versions, follow the existing workspace dependency policy.

### `space-client-tui`

Owns:

- terminal setup/restore,
- Ratatui rendering,
- keyboard handling,
- command-line editing,
- connection to server,
- local view model,
- reconnect/error display later.

Likely dependencies:

```toml
ratatui = "0.29"
crossterm = { version = "0.28", features = ["event-stream"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
tokio-tungstenite = "0.24"
serde_json = "1"
tui-input = "0.11"
tracing = "0.1"
color-eyre = "0.6"
space-game-protocol = { path = "../space-game-protocol" }
```

Use current repository-compatible versions rather than these exact versions if needed.

---

## Transport choice

Use WebSocket for the first networked slice.

Reasoning:

- It gives a persistent bidirectional connection.
- It is easy to run locally.
- It suits a TUI client receiving live updates while the user types.
- It keeps a future browser client possible.
- It is simpler than gRPC for a mixed event stream of commands, logs, prompts and snapshots.

Default endpoint:

```text
ws://127.0.0.1:4000/ws
```

The client should accept a configurable server URL eventually, but a hard-coded local default is fine for the first slice.

---

## Initial protocol sketch

The protocol should be simple and JSON-serialised for now.

Example shape:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientToServer {
    Hello {
        client_name: String,
        client_version: String,
    },
    Command {
        seq: u64,
        text: String,
    },
    RequestObjects {
        seq: u64,
    },
    RequestDistances {
        seq: u64,
        limit: Option<usize>,
        sort: DistanceSort,
    },
    RequestDistance {
        seq: u64,
        object_query: String,
    },
    Ping {
        seq: u64,
    },
}
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerToClient {
    Welcome {
        server_version: String,
        session_id: String,
    },
    CommandAck {
        seq: u64,
        accepted: bool,
        message: Option<String>,
    },
    Status {
        game_time: String,
        observer_label: String,
        object_count: usize,
    },
    Objects {
        seq: u64,
        objects: Vec<SolarSystemObjectSummary>,
    },
    Distance {
        seq: u64,
        result: DistanceResult,
    },
    Distances {
        seq: u64,
        results: Vec<DistanceResult>,
    },
    OutputLine {
        line: String,
    },
    Error {
        seq: Option<u64>,
        code: String,
        message: String,
    },
    Pong {
        seq: u64,
    },
}
```

Supporting types might look like:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarSystemObjectSummary {
    pub id: String,
    pub display_name: String,
    pub object_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceResult {
    pub object_id: String,
    pub display_name: String,
    pub distance_km: f64,
    pub distance_au: Option<f64>,
    pub at_game_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistanceSort {
    Name,
    Distance,
}
```

The exact timestamp type should be chosen after inspecting `space-game-ephemeris`. Prefer preserving a typed timestamp internally and converting to protocol-friendly display/serialisation at the edge.

---

## Command handling

For the first slice, command parsing can be deliberately simple.

Suggested flow:

```text
TUI user enters command text
  -> client sends ClientToServer::Command { seq, text }
  -> server parses command
  -> server sends CommandAck
  -> server sends one or more response messages
  -> client appends displayable lines to output log and updates status pane
```

The server, not the TUI client, should be the source of truth for command semantics.

However, the client can eventually use protocol-provided command metadata for autocomplete. Do not design the full autocomplete system yet. A minimal local autocomplete list for `help`, `objects`, `distance`, `distances`, `status`, `quit` is acceptable for the first TUI.

---

## TUI client design

The Ratatui client should keep a local view model:

```rust
pub struct ClientApp {
    pub connected: bool,
    pub server_url: String,
    pub output_lines: Vec<String>,
    pub status: ClientStatusView,
    pub command_input: CommandInputState,
    pub should_quit: bool,
}
```

The event loop should merge:

- terminal keyboard input,
- terminal resize events,
- render ticks,
- network messages from the server.

Conceptual loop:

```rust
loop {
    tokio::select! {
        maybe_input = terminal_events.next() => {
            handle_terminal_input(&mut app, maybe_input, &mut connection).await?;
        }
        maybe_msg = connection.recv() => {
            apply_server_message(&mut app, maybe_msg?);
        }
        _ = render_tick.tick() => {
            terminal.draw(|frame| draw(frame, &app))?;
        }
    }

    if app.should_quit {
        break;
    }
}
```

Rendering should use a stable OpenCode-like layout:

- top-left: output/event log,
- top-right: status panel,
- bottom: command input.

Keep rendering pure: `draw(frame, &app)` should not perform network calls or mutate game state.

---

## Server design

For the first slice, the server can be minimal:

```text
space-server
  main()
    initialise tracing
    initialise ephemeris service
    create fixed observer location
    start Axum server

  /ws handler
    accept WebSocket
    send Welcome
    send initial Status
    loop:
      receive ClientToServer JSON
      handle message
      send ServerToClient JSON responses
```

The server should have a small service layer between WebSocket handling and ephemeris calls:

```rust
pub struct SolarSystemQueryService {
    ephemeris: EphemerisModel,
    observer: ObserverLocation,
}

impl SolarSystemQueryService {
    pub fn list_objects(&self) -> Result<Vec<SolarSystemObjectSummary>, QueryError>;
    pub fn distance_to(&self, object_query: &str, at: GameTime) -> Result<DistanceResult, QueryError>;
    pub fn distances(&self, at: GameTime, sort: DistanceSort, limit: Option<usize>) -> Result<Vec<DistanceResult>, QueryError>;
}
```

The exact `EphemerisModel` and `GameTime` types should be adapted to the existing ephemeris crate.

If the ephemeris crate currently exposes a different API, the explore output should recommend the smallest adapter layer rather than forcing the crate into this exact shape.

---

## Distance calculation expectations

The implementation should:

1. Obtain the observer position in the same coordinate frame as the target object.
2. Obtain the target object position at the current game timestamp.
3. Calculate Euclidean distance.
4. Return distance in kilometres.
5. Also return astronomical units if that is straightforward.

Use a named constant for AU in kilometres if one does not already exist:

```rust
pub const AU_KM: f64 = 149_597_870.7;
```

The output should be clear that values are approximate and depend on the ephemeris model and timestamp.

---

## Minimal acceptance criteria

A successful first implementation should satisfy:

- `cargo check` passes for the workspace.
- `space-server` can start locally.
- `space-client-tui` can connect to the local server.
- The client renders an output pane, status pane and command input pane.
- The user can type `objects` and see a list of known objects.
- The user can type `distance <object>` and see the distance from the fixed observer location.
- The user can type `distances --limit 10` and see a list of distances.
- The status pane shows connection status, game timestamp, observer label and object count.
- The terminal is restored cleanly on exit or error.
- The server owns ephemeris queries; the client does not directly depend on `space-game-ephemeris`.

---

## Testing suggestions

Add tests at the most stable boundaries first.

### Protocol tests

- JSON round-trip for `ClientToServer` and `ServerToClient` messages.
- Backward-compatible naming conventions where practical.

### Query service tests

- Object list returns at least the expected core objects if the ephemeris model includes them.
- Distance calculation returns non-negative values.
- Unknown object query returns a displayable error.
- Limit and sort behaviour works for distance lists.

### Server tests, if cheap

- WebSocket accepts a connection.
- `Hello` receives `Welcome`.
- `RequestObjects` receives `Objects`.

### TUI tests

Keep these light initially.

- Test pure functions that apply server messages to `ClientApp`.
- Test command input handling where practical.
- Avoid complex terminal snapshot tests for the first slice unless the repository already uses them.

---

## Deliberate non-goals for this slice

Do not implement yet:

- real multiplayer gameplay,
- authentication,
- save games,
- custom stations/ships as persistent game objects,
- moving the observer location,
- orbital manoeuvres,
- command permissions,
- full autocomplete from server metadata,
- pop-up dialogs/wizards,
- binary protocols,
- gRPC,
- QUIC,
- ECS migration,
- browser client,
- production deployment.

Leave clean extension points, but do not let these concerns block the first working integration.

---

## Risks and questions for explore mode

Please inspect and answer these before proposing the final change set:

1. What public API does `space-game-ephemeris` currently expose?
2. Does it already have object IDs, object names and object types?
3. What timestamp/time representation does it expect?
4. What coordinate frame are positions returned in?
5. Are positions returned in kilometres, metres, AU or another unit?
6. Does it support all planets only, or planets plus moons/asteroids?
7. Is there already a workspace dependency/version policy?
8. Are there existing binary crates or CLI patterns to follow?
9. Should `space-server` and `space-client-tui` be separate binaries, separate crates, or binary targets in existing crates?
10. Is there an existing error-handling style such as `anyhow`, `thiserror` or `color-eyre`?

The implementation plan should adapt to the answers rather than assuming the sketch above is exact.

---

## Suggested first implementation steps

1. Inspect the current workspace and ephemeris crate API.
2. Add or identify a shared `space-game-protocol` crate.
3. Define minimal protocol messages and DTOs.
4. Add a small server-side query adapter around `space-game-ephemeris`.
5. Add `space-server` with a local WebSocket endpoint.
6. Add `space-client-tui` with the Ratatui layout and connection loop.
7. Wire commands to protocol messages and display responses.
8. Add basic tests for protocol and query service logic.
9. Add README/dev instructions for running the server and client locally.

Expected local demo:

```bash
cargo run -p space-server
cargo run -p space-client-tui
```

Then in the TUI:

```text
objects
distance mars
distances --limit 10
status
quit
```
