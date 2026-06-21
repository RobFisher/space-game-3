## 1. Manifest Model

- [ ] 1.1 Replace or extend the existing kernel manifest structs with profile-based ephemeris asset manifest structs.
- [ ] 1.2 Parse `version`, `profiles`, and `assets` TOML tables, including asset fields for kind, filename, source, URL, local path, optional checksum, optional exact size, optional approximate size, required flag, description, and source or licence notes.
- [ ] 1.3 Validate required fields, supported asset kinds, profile asset references, duplicate profile asset references, and unsafe local paths.
- [ ] 1.4 Add profile selection APIs that return selected assets or clear validation errors for unknown profiles or asset ids.

## 2. Asset Paths and Verification

- [ ] 2.1 Add asset root resolution using repo-root `data/ephemeris/` by default and `SPACE_GAME_EPHEMERIS_DATA_DIR` when set.
- [ ] 2.2 Add path resolution that joins validated manifest `local_path` values to the resolved asset root.
- [ ] 2.3 Implement offline verification for missing files, exact size mismatches, and checksum mismatches when checksum metadata is available.
- [ ] 2.4 Ensure verification errors include the asset id and resolved filesystem path.

## 3. Helper Command

- [ ] 3.1 Add an explicit `ephemeris-assets` helper binary or equivalent dev command with `list`, `verify`, and `fetch` subcommands.
- [ ] 3.2 Implement `list --profile <name>` to display selected asset ids, descriptions, URLs, and resolved local paths without downloading.
- [ ] 3.3 Implement `verify --profile <name>` to run offline verification and report actionable missing or invalid asset messages.
- [ ] 3.4 Implement `fetch --profile <name>` to download missing or invalid selected assets, verify them, and leave valid existing files unchanged unless forced.

## 4. Repository Data and Documentation

- [ ] 4.1 Add checked-in `data/ephemeris/manifest.toml` with minimal and inner profiles and the starting assets for `de442s`, `de442`, `mar099s`, and `pck11`.
- [ ] 4.2 Update `.gitignore` so downloaded ephemeris kernels under `data/ephemeris/kernels/` are not tracked.
- [ ] 4.3 Document the asset helper commands, default asset root, environment override, and no-network test expectation in project documentation.

## 5. Tests and Validation

- [ ] 5.1 Add fixture-based tests for valid manifest parsing, invalid manifest rejection, profile selection, and unsafe path rejection.
- [ ] 5.2 Add fixture-based tests for asset root/path resolution and offline verification success and failure cases.
- [ ] 5.3 Add helper command tests or lower-level fetch tests that use local or mocked sources rather than internet access.
- [ ] 5.4 Run relevant Cargo tests and `openspec validate --all`.
