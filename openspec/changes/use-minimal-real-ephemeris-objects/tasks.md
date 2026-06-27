## 1. Pre-implementation Checks

- [x] 1.1 Review the active change artifacts and make a focused git commit preserving the pre-implementation OpenSpec state before application code changes.
- [ ] 1.2 Confirm local `minimal` profile assets verify successfully, and document the expected failure path for environments where they are missing.

## 2. Ephemeris Backend

- [ ] 2.1 Select and add the Rust-native SPICE/ANISE dependency needed to read the checked-in manifest's local `de442s` and `pck11` assets.
- [ ] 2.2 Route `SolarSystem::state` for `EphemerisSource::SpiceBody` through a private provider that loads configured local kernels and returns game-facing `StateVector` values.
- [ ] 2.3 Keep SPICE/ANISE types private to `space-game-ephemeris` and preserve the existing public API shape.
- [ ] 2.4 Return clear `KernelNotFound`, backend, or `OutOfCoverage` errors for missing, invalid, unconfigured, or out-of-coverage SPICE body queries without downloading files.
- [ ] 2.5 Cache or otherwise reuse loaded kernel context so repeated state queries do not reopen kernels unnecessarily.
- [ ] 2.6 Add ephemeris tests for successful SPICE body resolution when local fixture or profile assets are available, and for missing-kernel and out-of-coverage error behavior.

## 3. Default Game Registry

- [ ] 3.1 Replace placeholder planet and moon entries in the default server registry with `SpiceBody` entries for Sun, Mercury, Venus, Earth, Moon, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto using manifest NAIF ids.
- [ ] 3.2 Remove placeholder-only objects that are not in the `minimal` profile, including Ceres and the `luna` alias.
- [ ] 3.3 Keep fictional station entries and ensure their parent references point to real registered celestial objects.
- [ ] 3.4 Configure the default server world builder to use the checked-in ephemeris manifest and resolved asset root needed for the `minimal` profile.
- [ ] 3.5 Update server query and command tests for the new default object list, real-body source behavior, station parentage, and missing-asset failures.

## 4. Player Ship Integration

- [ ] 4.1 Ensure the default player ship still orbits object id `earth`, now resolved from the real default registry.
- [ ] 4.2 Update player ship and flight-plan tests to assert real Earth parentage while preserving mutable ship-state semantics.

## 5. Docs and Validation

- [ ] 5.1 Update README text and examples that describe fictional solar-system objects so they explain the real `minimal` profile requirement and explicit asset fetch flow.
- [ ] 5.2 Run targeted tests for `space-game-ephemeris` and `space-server`.
- [ ] 5.3 Run relevant plain-client smoke checks against the local server when local `minimal` assets are available.
- [ ] 5.4 Run `openspec validate --all`.
- [ ] 5.5 Commit the completed implementation, tests, docs, and updated OpenSpec task checkboxes in focused chunks.
