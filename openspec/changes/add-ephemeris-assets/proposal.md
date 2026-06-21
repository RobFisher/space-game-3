## Why

Real solar-system ephemeris files are currently expected to be handled manually, which makes local setup fragile and unclear for open source development. The project needs a checked-in manifest and explicit helper commands so required data files can be discovered, fetched, and verified without network access during normal builds or tests.

## What Changes

- Define a profile-based ephemeris asset manifest format for real kernel/data files.
- Add a repo-root default asset location under `data/ephemeris/`, with an environment variable override for developers and CI jobs that keep data elsewhere.
- Add explicit asset helper commands to list, verify, and fetch selected profile assets.
- Validate local paths, required fields, file sizes, and checksums where available.
- Fail clearly when required selected assets are missing or invalid.
- Keep normal library compilation, unit tests, and default server startup free of implicit downloads.
- Add a checked-in starting manifest for useful real solar-system assets, while keeping large downloaded files out of git.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `ephemeris-core`: Extend the kernel manifest model into a profile-based ephemeris asset manifest and define explicit offline verification and opt-in download behavior.

## Impact

- Updates `crates/space-game-ephemeris` manifest parsing and validation APIs.
- Adds an explicit ephemeris asset helper binary or equivalent dev command.
- Adds a checked-in `data/ephemeris/manifest.toml` and `.gitignore` rules for downloaded asset files.
- May add HTTP, checksum, and CLI parsing dependencies for the helper command, kept out of normal library behavior where practical.
- Adds fixture-based tests that do not require internet access or large ephemeris files.
