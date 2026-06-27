## Why

The game can now list celestial objects covered by downloaded ephemeris assets, but the running server still exposes placeholder circular-orbit planets and moons. Moving the default game registry to the existing `minimal` profile makes object lists, distances, locations, stations, and flight plans operate against the real celestial identities already described by the asset manifest.

## What Changes

- Replace the server's placeholder planet and moon registry entries with the celestial objects covered by valid local `minimal` profile assets: Sun, Mercury, Venus, Earth, Moon, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto.
- Remove placeholder-only bodies that are not in the `minimal` profile, including Ceres and the `luna` alias.
- Keep fictional stations, but parent their positions or orbits to real registered celestial objects.
- Implement enough SPICE/ANISE-backed body state resolution for configured local profile assets so default server distance and location queries complete for the real bodies without network access.
- Keep ephemeris asset downloads explicit; server startup and normal queries must not download missing kernels.
- Update the player ship's default orbit to remain around the real Earth object.
- Update docs and tests that currently describe or assert fictional demo planets and moons.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `ephemeris-core`: SPICE-backed body sources resolve from configured local ephemeris assets instead of always returning an unimplemented-backend error.
- `authoritative-space-server`: The default server registry uses real `minimal` profile celestial objects plus fictional stations attached to those objects instead of placeholder planet and moon definitions.
- `player-ship-state`: The default ship orbit is anchored to the real Earth registry object while remaining mutable server-owned ship state.

## Impact

- Affected crates: `crates/space-game-ephemeris`, `crates/space-server`, and tests that construct default worlds or assert object lists.
- Affected data: `crates/space-server/data/demo_registry.toml` or a renamed equivalent default registry file.
- Affected docs/specs: README and OpenSpec requirements that still describe fictional demo solar-system bodies.
- New dependency risk: the ephemeris crate will likely need a Rust-native SPICE/ANISE dependency and associated time/kernel loading glue.
- Runtime behavior: default server queries will require valid local `minimal` profile assets and will fail clearly when required assets are missing or invalid, without attempting downloads.
