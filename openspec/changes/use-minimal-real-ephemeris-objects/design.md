## Context

The ephemeris asset manifest already defines a `minimal` profile that selects `de442s` and `pck11`. Local verification currently succeeds, and the helper can list the valid downloaded celestial objects from `de442s`: Sun, Mercury, Venus, Earth, Moon, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto.

The running server still builds its default world from `crates/space-server/data/demo_registry.toml`, where planets and the moon are authored as fictional static or circular-orbit objects. The ephemeris crate has an `EphemerisSource::SpiceBody` variant, but `SolarSystem::state` routes through `resolution::resolve_global_state`, which currently returns an unimplemented backend error for every SPICE body.

This change crosses the ephemeris crate, server default data, player ship state, tests, and docs. It should preserve the existing server/client protocol shape: clients still ask for objects, distances, locations, flight plans, and ship status through the server.

## Goals / Non-Goals

**Goals:**

- Make the default server object registry use the real celestial objects covered by valid local `minimal` profile assets.
- Remove placeholder planets and moons from the default game registry, including Ceres and `luna`.
- Keep fictional stations and resolve them relative to real celestial parent objects.
- Resolve `SpiceBody` states from local configured assets without exposing SPICE or ANISE types through the public game-facing API.
- Fail clearly when required local assets are missing or invalid, and never download assets during server startup or normal queries.
- Keep the player ship as mutable server-owned state orbiting the real Earth object by default.

**Non-Goals:**

- Implement body-fixed surface locations or planetary orientation transforms.
- Add Mars moons from the `inner` profile.
- Add every body present in larger kernels or introduce dynamic profile selection at runtime.
- Change the client/server protocol DTOs.
- Persist stations, ship state, or profile selection beyond the current server configuration.

## Decisions

### Use `minimal` profile coverage as the default real-body set

The default game registry should contain the object identities listed from valid local `minimal` profile assets: `sun`, `mercury`, `venus`, `earth`, `moon`, `mars`, `jupiter`, `saturn`, `uranus`, `neptune`, and `pluto`.

Rationale: this matches the asset manifest source of truth and avoids hand-maintaining a separate fictional body list. It also keeps scope bounded: Phobos and Deimos remain outside this change because they are selected by the `inner` profile, not `minimal`.

Alternative considered: keep placeholder circular-orbit bodies and only rename `luna` to `moon`. That would preserve tests but would not make the game use the ephemeris assets the user can now inspect.

### Represent real bodies as `SpiceBody` registry objects

Each real celestial object should be an `EphemerisSource::SpiceBody` with the NAIF id from `data/ephemeris/manifest.toml`. The public `SolarSystem` API should continue returning `StateVector` with `FrameId::SolarSystemBarycentricJ2000` and `EphemerisQuality::RealKernel`.

Rationale: `SpiceBody` already exists as the intended source variant, and keeping the API stable lets server query code, command handling, and protocol DTOs remain unchanged.

Alternative considered: generate sampled trajectories from the kernels and store them as game-authored data. That would avoid a runtime SPICE dependency but creates stale derived data, larger repo artifacts, and a second source of truth.

### Load local kernels explicitly, never implicitly fetch

The server should build its world with the checked-in manifest and resolved asset root. It may verify required `minimal` assets before serving queries or lazily fail on the first real-body query, but errors must name the missing or invalid asset and the path. It must not fetch missing kernels.

Rationale: the existing asset helper already makes downloads explicit, and tests must avoid network access.

Alternative considered: automatically fetch `minimal` assets during server startup. That would be surprising runtime network behavior and conflicts with the existing "no implicit downloads" asset contract.

### Keep fictional objects as children of real parents

Fictional stations should stay as game-authored registry objects using existing supported parent-relative source types such as `fixed_offset` or `circular_orbit`, with parents that are real celestial objects. `demo-station` can remain parented to `earth`.

Rationale: this keeps the game flavor and existing object/query workflows while making the spatial reference real. Parent-child composition already combines parent state with local fictional offsets.

Alternative considered: remove stations until full real ephemeris support is complete. That would narrow the change but lose an existing gameplay affordance the user explicitly wants to keep.

### Keep the player ship outside the object registry

The default player ship should continue to be mutable server state, not a registry object. Its default orbit parent remains object id `earth`, now resolved through the real registry entry.

Rationale: this preserves current ship commands, object lookup semantics, and flight-plan behavior while improving the underlying parent state.

Alternative considered: register the ship as another ephemeris object. That would blur mutable gameplay state with the public object registry and conflicts with the existing player ship state contract.

## Risks / Trade-offs

- Rust SPICE/ANISE API mismatch or kernel support gaps -> Do a focused implementation spike inside `space-game-ephemeris`, keep ANISE/SPICE types private, and add tests that can run against fixture assets or skip real-kernel assertions when local kernels are unavailable.
- Default server startup may fail for users without downloaded kernels -> Produce a clear error that includes `cargo run -p space-game-ephemeris --bin ephemeris-assets -- fetch --profile minimal`, and document the requirement.
- Real ephemeris coverage may not include the configured default game time -> Check coverage during implementation and either adjust `DEFAULT_GAME_TIME` to an in-coverage timestamp or report `OutOfCoverage` with the queried object and timestamp.
- Query performance may degrade if kernels are opened for every state call -> Cache loaded kernel context in `SolarSystem` or a private provider object.
- Existing tests assume fictional deterministic circular positions -> Update tests to assert object membership, source configuration, error behavior, and station composition rather than exact placeholder positions unless using controlled fixtures.

## Migration Plan

1. Add the private SPICE/ANISE-backed resolution path behind the existing `SpiceBody` source variant.
2. Replace the default server registry bodies with real `minimal` profile bodies and keep fictional stations attached to real parents.
3. Update player ship tests and command/query tests for `moon` instead of `luna` and remove Ceres/default fictional body expectations.
4. Update README and specs to describe real minimal-profile object data and explicit asset requirements.
5. Validate with targeted crate tests, server command tests, and `openspec validate --all`.

Rollback is a code/data revert to the previous placeholder registry and unimplemented SPICE behavior. No persisted data migration is required.

## Open Questions

- Which Rust SPICE/ANISE API version should be used after checking currently available crate support in the implementation environment?
- Is the existing default game time `2097-01-01T00:00:00Z` inside `de442s` coverage for every `minimal` object, or should the default simulation time move to a known supported epoch?
