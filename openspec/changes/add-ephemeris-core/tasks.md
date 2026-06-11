## 1. Crate Structure and Dependencies

- [ ] 1.1 Add crate dependencies for error handling, serialization, TOML parsing, and time handling.
- [ ] 1.2 Replace the stub library with module declarations for time, vector, state, object, source, registry, manifest, providers, world, resolution, and error handling.
- [ ] 1.3 Re-export the intended public API from `lib.rs` while keeping backend/provider internals private where practical.

## 2. Core Data Types

- [ ] 2.1 Implement `EphemerisError` variants for unknown objects, missing kernels, out-of-coverage queries, unavailable frame transforms, invalid definitions, cyclic dependencies, and backend failures.
- [ ] 2.2 Implement `GameTime` with ISO-8601 UTC parsing, ordering, duration/difference helpers, and serialization support needed by registry data.
- [ ] 2.3 Implement `Vec3Km`, `Vec3KmPerSec`, arithmetic helpers, distance calculation, and finite-value validation.
- [ ] 2.4 Implement `FrameId`, `EphemerisQuality`, and `StateVector` with helpers for combining parent and local states.
- [ ] 2.5 Implement `ObjectId`, `ObjectKind`, `PhysicalProperties`, `GameplayMetadata`, `ObjectDefinition`, and `ObjectSummary`.

## 3. Registry and Manifest Loading

- [ ] 3.1 Implement `EphemerisSource` serialization for fixed offset, circular orbit, sampled trajectory, SPICE body, and body-fixed source variants.
- [ ] 3.2 Implement TOML object registry loading from strings and paths.
- [ ] 3.3 Validate registry uniqueness, required source fields, finite numeric values, circular orbit radius/period constraints, and implemented source parent references.
- [ ] 3.4 Implement metadata lookup and object listing from the registry.
- [ ] 3.5 Implement kernel manifest data structures and TOML parsing without any network access or download behavior.
- [ ] 3.6 Validate manifest schema version, kernel ids, kernel kinds, filenames, URLs, checksums, and required flags.

## 4. State Resolution

- [ ] 4.1 Implement `SolarSystemBuilder` with optional registry path, manifest path, kernel directory, and approximate fallback configuration fields.
- [ ] 4.2 Implement `SolarSystem::state`, `state_relative_to`, `position`, `distance`, `light_time_seconds`, `list_objects`, and `object_metadata`.
- [ ] 4.3 Implement recursive global-state resolution with visited-set cycle detection.
- [ ] 4.4 Implement fixed-offset resolution by adding parent and local state.
- [ ] 4.5 Implement circular orbit position and velocity calculation with inclination, RAAN, phase, elapsed time, and parent-state addition.
- [ ] 4.6 Implement sampled trajectory exact-sample lookup, linear interpolation, parent-state addition, and out-of-range errors.
- [ ] 4.7 Implement explicit unsupported behavior for SPICE body and body-fixed source queries.

## 5. Provider Boundary

- [ ] 5.1 Define an internal provider abstraction suitable for future ANISE-backed state resolution.
- [ ] 5.2 Implement a game-object provider for pure authored sources.
- [ ] 5.3 Implement a stub SPICE provider that returns clear backend or missing-kernel errors without exposing ANISE/SPICE types.

## 6. Tests and Validation

- [ ] 6.1 Add unit tests for `GameTime`, vector math, state combination, distance, and light-time calculations.
- [ ] 6.2 Add registry loading tests for valid registries, duplicate ids, metadata preservation, and invalid source definitions.
- [ ] 6.3 Add state resolution tests for fixed offsets, dependency cycles, circular orbit periodicity, tangential velocity, sampled trajectory interpolation, exact samples, and out-of-range queries.
- [ ] 6.4 Add manifest parsing and validation tests, including a no-network expectation.
- [ ] 6.5 Run `cargo test -p space-game-ephemeris`.
- [ ] 6.6 Run `openspec validate --all`.
