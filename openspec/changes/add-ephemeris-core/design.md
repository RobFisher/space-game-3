## Context

`crates/space-game-ephemeris` currently exposes only a stub function. The design note in `crates/space-game-ephemeris/design_notes.md` describes a larger destination: one game-facing API over real SPICE-backed Solar System bodies and authored gameplay objects such as stations, ships, bases, and scripted routes.

This change establishes the first implementation boundary. It delivers useful pure-Rust ephemeris behavior for authored objects and creates stable seams for later ANISE/SPICE and downloader work without requiring real kernels now.

## Goals / Non-Goals

**Goals:**

- Provide a game-facing API for object state, relative state, distance, and light-time queries.
- Model object identity, frames, state vectors, quality metadata, physical metadata, gameplay metadata, and errors.
- Load object definitions from a checked-in or caller-provided TOML registry.
- Resolve fixed offsets, circular orbits, and sampled trajectories using pure math.
- Detect unknown objects, cyclic dependencies, invalid definitions, and sampled trajectory coverage failures.
- Define a kernel manifest model suitable for future verified downloads and backend loading.
- Define a provider boundary that can wrap ANISE later without exposing ANISE types publicly.

**Non-Goals:**

- Do not implement real SPICE/ANISE kernel loading in this change.
- Do not implement kernel downloading, verification commands, or third-party notice generation.
- Do not implement body-fixed frame transforms beyond parsing and explicit unsupported errors.
- Do not implement Kepler orbits, Lagrange points, small-body SPK generation, or a CLI.
- Do not make game runtime perform network access.

## Decisions

### Keep the public API independent of ANISE/SPICE

The crate will expose `SolarSystem`, `SolarSystemBuilder`, `GameTime`, `ObjectId`, `FrameId`, `StateVector`, registry types, and error types. A private or crate-scoped provider trait will separate public queries from backend implementation.

Alternative considered: expose ANISE almanac, frame, or epoch types directly. That would speed up the later SPICE integration but would couple the TUI game and authored objects to backend-specific concepts. The design note explicitly calls for hiding ANISE so it can be replaced later.

### Use one object registry for real and authored objects

The registry will store `ObjectDefinition` values keyed by string `ObjectId`. Each object has an `EphemerisSource` describing whether it is a SPICE body, fixed offset, circular orbit, sampled trajectory, body-fixed location, or a future source type.

Alternative considered: separate registries for real bodies and game objects. That makes provider dispatch simpler but leaks source categories into gameplay code and makes mixed queries like station-to-moon distance less uniform.

### Implement pure gameplay sources first

This change implements:

- `StaticState`
- `FixedOffset`
- `CircularOrbit`
- `SampledTrajectory` with linear interpolation

These sources can be tested without kernels and are enough to model early fictional stations, routes, mock parents, inertial fixture bodies, and deterministic gameplay fixtures.

Alternative considered: start with ANISE real-body integration. That creates immediate value for planets, but it depends on kernel asset choices, text-kernel conversion behavior, and downloader decisions that are still better handled in a later, focused change.

### Use a small custom vector type publicly

The public API will use `Vec3Km` and `Vec3KmPerSec` with `f64` fields and basic arithmetic helpers. This avoids committing callers to `glam`, `nalgebra`, or ANISE math types.

Alternative considered: use `glam` because it is common in games. `glam` is optimized for real-time graphics, but this crate needs double-precision ephemeris values and a stable data model more than SIMD vector ergonomics.

### Hide precise time machinery behind `GameTime`

`GameTime` will expose ISO-8601 UTC parsing and duration/difference helpers needed by orbit and interpolation math. Internally it can use `hifitime::Epoch` or another precise representation, but callers should not need to reason about ET/TDB conversions in this phase.

Alternative considered: use `chrono` or raw seconds since Unix epoch publicly. Raw seconds are simple but make future SPICE conversions and leap-second behavior harder to represent correctly.

### Return explicit quality metadata

Every returned `StateVector` will include `EphemerisQuality`, such as fictional, approximate, or real-kernel placeholder values. In this change, pure gameplay sources return fictional or approximate quality, while SPICE bodies return a clear unsupported or missing-backend error.

Alternative considered: omit quality until real kernels exist. Adding it later would force API churn and would make it harder for UI/gameplay code to distinguish authored positions from future real ephemeris positions.

### Represent kernel manifests now, fetch kernels later

This change adds manifest structs for kernel metadata: id, kind, filename, URL, checksum, size, required flag, coverage notes, and profile/schema fields. It does not download or validate files yet.

Alternative considered: delay manifest modeling until downloader work. Defining the data model now helps the provider boundary and config API settle while avoiding network behavior during normal builds.

## Risks / Trade-offs

- Pure gameplay math may initially be less physically complete than full orbital mechanics -> Keep supported sources intentionally small and test their documented behavior thoroughly.
- `GameTime` internals may need adjustment when ANISE integration starts -> Keep conversion helpers encapsulated and avoid exposing backend time types publicly.
- The provider trait may need refinement once real ANISE APIs are wired in -> Keep it crate-private where possible and expose only stable game-facing methods.
- Body-fixed objects may parse but not resolve in this phase -> Return `FrameTransformUnavailable` with a clear message rather than silently approximating.
- Registry validation can become too permissive if future source variants are stubbed -> Validate required fields and parent references for implemented source types, and return explicit unsupported errors for parsed but unresolved variants.

## Migration Plan

This is the first substantive implementation of the crate, so there is no runtime migration. Existing callers of `crate_name()` are test-only and can be removed or replaced as the public API takes shape.

Implementation should remain isolated to `crates/space-game-ephemeris` plus optional test fixtures or crate-local sample data. Later changes can build on these APIs for downloader, ANISE provider, and CLI work.

## Open Questions

- Should `GameTime` expose any feature-gated conversion method for backend integrations, or should that remain entirely crate-private?
- Should the kernel manifest live under `data/kernels/` in this first change, or should this change only define parser/types and leave checked-in manifest files to the downloader change?
- Should `BodyFixed` be included as a parsed source variant immediately, or postponed entirely until frame transform work begins?
