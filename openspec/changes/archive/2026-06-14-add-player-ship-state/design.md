## Context

The server currently owns a `SolarSystem`, a simulation clock, and a fixed `ObserverLocation`. Query behavior uses that observer as the origin for distance calculations, `where` without arguments, and status display. The protocol and TUI expose the same observer-centered vocabulary through `StatusDto`, the status pane, and plain text status output.

The planned design calls for the next gameplay subject to be the player ship. This change introduces that subject without adding persistence, authentication, multiplayer identity, flight planning, or a real thrust integrator.

## Goals / Non-Goals

**Goals:**

- Introduce one server-owned player ship with stable id, editable display name, motion mode, and state resolution at a `GameTime`.
- Default the player ship to a fictional orbit near Earth.
- Use the player ship as the subject for `status`, `where`, `distance`, and `distances`.
- Let the TUI and plain text client display the ship name in status output.
- Keep ship state authoritative on the server and protocol DTOs independent of server and ephemeris implementation crates.

**Non-Goals:**

- Multiplayer identity, per-client ships, authentication, or ownership.
- Persistence of ship name or motion state across server restarts.
- Flight planning, navigation UI, or planned maneuvers.
- Continuous thrust propagation or realistic orbital mechanics beyond the initial fictional orbit mode.
- Adding the player ship to the public object registry or object autocomplete list.

## Decisions

### Represent the player ship as server game state

The server will introduce a `PlayerShip` or equivalent model alongside the `SolarSystemQueryService` state instead of adding the ship to `demo_registry.toml`.

Rationale: the object registry represents known ephemeris/demo objects, while the ship is mutable game state. Keeping the ship outside the registry avoids making object listing, object completion, and future persistence semantics ambiguous.

Alternative considered: add `player-ship` as a demo registry object. That would allow reuse of `SolarSystem::state`, but it would mix mutable player state into static registry data and make renaming awkward.

### Resolve ship state through a motion-mode abstraction

The initial motion mode will be `Orbiting`, configured with parent object id, radius, period, orbital angles, and epoch. Resolving the ship state at time `t` will compute a parent-relative circular orbit and add the parent object's resolved state, mirroring the existing fictional circular-orbit behavior.

The model should have a place for future `UnderThrust` or similar modes, but this change will only construct and resolve `Orbiting`.

Rationale: the user explicitly needs the ship to behave like a fictional ephemeris object while in orbit and have room for other behavior under thrust later. A motion-mode enum gives that extension point without implementing flight planning now.

Alternative considered: keep a fixed state vector and update it only when commands run. That would be simpler but would fail the "ephemeris-like while in orbit" requirement because advancing simulation time would not move the ship.

### Replace observer semantics at the query boundary

Distance and location query code will use the current player ship state where it previously used `ObserverLocation`. `where` without arguments will report the player ship with `subject_type = "ship"`. Object-specific `where <object>` behavior remains object-centered.

Rationale: this preserves existing command ergonomics while changing the gameplay subject from an abstract observer to the player's ship.

Alternative considered: add only new `ship where` and `ship distance` commands while leaving existing commands observer-centered. That would keep backward compatibility but maintain two origins for the first slice, creating confusing status and distance behavior.

### Add explicit ship commands for status and naming

The server will accept `ship`, `ship status`, and `ship name <name>`. The first two return current ship state/status. The rename command updates the in-memory ship display name and returns updated ship state/status. Empty names are rejected.

Rationale: `status` remains the high-level connection/game status command, while `ship` gives a direct way to inspect and name the gameplay subject.

Alternative considered: use `name ship <name>`. `ship name <name>` keeps future ship subcommands grouped under one command prefix.

### Evolve status DTOs to ship fields

`StatusDto` will expose ship-oriented fields such as `ship_id`, `ship_name`, `ship_frame`, and `ship_motion`. The TUI view model should use these for the status pane. Backward-compatible observer aliases are not required because this repository controls both protocol ends.

Rationale: the wire contract should describe the current gameplay model directly. Carrying observer names forward would make the next layer of UI copy and tests misleading.

Alternative considered: keep `observer_label` and store the ship name there. That minimizes edits but bakes obsolete vocabulary into the protocol.

## Risks / Trade-offs

- Protocol field rename breaks old clients -> acceptable for this local first-slice project; update server, TUI, plain mode, and tests in the same change.
- Circular orbit implementation may duplicate ephemeris code -> keep it small for now; extract shared orbit math later only if duplication becomes meaningful.
- Ship state outside the object registry means `objects` will not list the ship -> document through specs and keep ship commands explicit until game entities and persistence are designed.
- In-memory naming resets on restart -> acceptable because persistence is out of scope; tests should assert runtime behavior only.

