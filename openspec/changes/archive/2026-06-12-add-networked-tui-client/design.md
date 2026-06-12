## Context

The repository currently contains a Cargo workspace with a top-level `space-game` binary and a reusable `space-game-ephemeris` crate. The ephemeris crate already exposes a game-facing `SolarSystem` API with object listing, object state lookup, positions, distances between registered objects, typed `GameTime`, `FrameId`, and fictional source types such as static state, circular orbit, sampled trajectory, and fixed offset.

This change introduces the first networked vertical slice: a local authoritative server owns ephemeris access, a shared protocol crate defines wire-visible JSON messages, and a Ratatui client connects over WebSocket to list objects and query distances. The implementation should remain intentionally small and avoid designing final multiplayer, persistence, or simulation systems.

## Goals / Non-Goals

**Goals:**

- Add a minimal shared protocol that can be serialized as JSON over WebSocket.
- Add a local authoritative server that serves object, distance, distances, status, help, and ping interactions.
- Use a fictional demo registry built from currently supported ephemeris source variants.
- Keep the fixed observer as server-owned state rather than a client-side or ephemeris-core concept.
- Add a Ratatui client with an output log, independently updated status pane, and command input.
- Preserve the boundary that the client does not depend directly on `space-game-ephemeris`.

**Non-Goals:**

- No persistence, authentication, accounts, combat, trading, navigation, or full simulation scheduling.
- No SPICE kernel download or body-fixed transform work.
- No production-grade command language, autocomplete protocol, reconnect policy, or browser client.
- No requirement to make `crates/space-game` the final integration binary in this slice.

## Decisions

### Add dedicated protocol, server, and TUI crates

The workspace will add:

```text
crates/space-game-protocol
crates/space-server
crates/space-client-tui
```

The existing `space-game` crate can remain as a placeholder. Separate crates make the architectural boundary explicit and keep the TUI from accidentally importing the ephemeris library.

Alternative considered: put server and TUI modules under `space-game`. That would reduce crate count but make dependency boundaries less clear for the first client/server slice.

### Use JSON messages over WebSocket

The first transport will be WebSocket at a local default endpoint:

```text
ws://127.0.0.1:4000/ws
```

Messages will be JSON-serialized protocol enums. WebSocket keeps a persistent bidirectional stream suitable for command responses and status updates while remaining simple enough for a local demo.

Alternative considered: line-delimited TCP. It would be simple, but WebSocket leaves a future browser client path open and is well-supported by Axum and TUI client libraries.

### Server owns command semantics

The TUI will send user-entered command text to the server as `ClientToServer::Command { seq, text }`. The server parses commands such as `help`, `objects`, `distance <object>`, `distances`, `distances --limit <n>`, `distances --sort distance`, and `status`.

The protocol may also include typed request messages for tests and future clients, but the first TUI command behavior should be accepted by the server. This keeps the server authoritative over game-visible operations.

Alternative considered: parse commands entirely in the TUI and send only typed requests. That would make the first client simpler to test locally but move game command semantics into the client.

### Use a fictional demo registry file

The server will load or embed a small demo object registry using currently implemented ephemeris source types:

```text
sun           static_state
mercury       circular_orbit around sun
venus         circular_orbit around sun
earth         circular_orbit around sun
mars          circular_orbit around sun
ceres         circular_orbit around sun
luna          circular_orbit around earth
demo-station  fixed_offset from earth
```

The registry should be data-driven, preferably committed as a TOML file under the server crate, so the server proves the existing registry loader path.

Alternative considered: construct `ObjectDefinition` values directly in Rust. That would avoid a data file but would not exercise the existing TOML registry loading path.

### Keep the fixed observer in the server adapter

The first observer will be a hard-coded server-side location:

```text
label: demo-observer
frame: SolarSystemBarycentricJ2000
position: [149_597_870.7, 0.0, 0.0] km
```

Distance queries will resolve target object position through `SolarSystem::state` or `SolarSystem::position`, then calculate Euclidean distance from the observer position. This avoids changing `space-game-ephemeris` for an arbitrary Cartesian observer API before the need is proven.

Alternative considered: add the observer as a registered ephemeris object. That would allow `SolarSystem::distance(observer, target, time)` but would blur the distinction between demo query configuration and actual game objects.

### Keep protocol DTOs separate from ephemeris types

Protocol types should use stable wire-visible strings and scalar values, such as object id, display name, kind, distance kilometers, distance AU, game time string, observer label, and frame label. Server code adapts from ephemeris types to protocol DTOs.

Alternative considered: reuse ephemeris structs directly in protocol messages. That is faster initially but couples the wire format to internal domain types.

## Risks / Trade-offs

- Demo data can be mistaken for real ephemeris accuracy -> Status and output should make the fictional/demo nature clear through labels and quality fields.
- Object lookup is ID-only in the registry today -> The server should implement a small case-insensitive resolver over object ids and display names for command input.
- Unsupported ephemeris source types can fail at query time -> The demo registry should avoid SPICE and body-fixed sources, and server errors should surface clear protocol `Error` messages.
- Async terminal and network event handling can leave the terminal in a bad state on panic or error -> The TUI should centralize terminal setup/restore and test command/message handling separately from terminal rendering where practical.
- WebSocket crate version compatibility may need adjustment -> Use versions compatible with the current Rust toolchain and dependency graph rather than treating brief versions as mandatory.

## Migration Plan

This is an additive change. Add the new crates to the workspace, introduce new dependencies, and keep existing ephemeris behavior unchanged. Rollback is removal of the new crates and workspace entries.

## Open Questions

- Should `space-game` eventually wrap or launch the TUI client, or should `space-client-tui` remain the primary runnable binary for now?
- Should the default observer be at the solar-system barycenter or at a demo 1 AU position? This design chooses the 1 AU position to avoid `distance sun` being zero in the first demo.
