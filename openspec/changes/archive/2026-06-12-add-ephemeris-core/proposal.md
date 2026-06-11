## Why

The game needs one stable ephemeris API that can answer position, velocity, distance, and light-time questions for both real Solar System bodies and authored gameplay objects. The current `space-game-ephemeris` crate is only a stub, while `crates/space-game-ephemeris/design_notes.md` captures enough direction to define the first useful implementation boundary.

## What Changes

- Add the initial game-facing ephemeris core to `crates/space-game-ephemeris`.
- Introduce public types for object identity, game time, frames, vectors, state vectors, quality, metadata, and errors.
- Add a TOML object registry for authored objects and real-body placeholders.
- Support pure local resolution for fixed offsets, circular orbits, and sampled trajectories with linear interpolation.
- Add recursive parent-child state resolution, cycle detection, relative state, distance, and light-time helpers.
- Add kernel manifest data structures so future kernel download and SPICE integration work has a stable configuration format.
- Add a SPICE provider boundary that keeps ANISE/SPICE types out of the game-facing API, but does not yet implement real kernel-backed state queries.
- Add focused tests for pure math, registry loading, dependency resolution, interpolation, and error cases.

## Capabilities

### New Capabilities

- `ephemeris-core`: Game-facing ephemeris API, object registry, pure gameplay object state resolution, kernel manifest model, and backend abstraction for future real-body providers.

### Modified Capabilities

- None.

## Impact

- Affected crate: `crates/space-game-ephemeris`.
- New dependencies are expected for serialization, TOML parsing, error handling, and time handling.
- The public Rust API of `space_game_ephemeris` will expand from the current stub into the core library surface.
- Real SPICE/ANISE kernel loading, kernel downloading, body-fixed frame transforms, and CLI tools remain out of scope for this change except where represented by stable boundaries or data models.
