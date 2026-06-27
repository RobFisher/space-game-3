## Why

Developers can list and verify ephemeris asset files, but they cannot ask which celestial objects are available from the files currently downloaded for a profile. This makes it harder to understand what local ephemeris data is usable without reading kernel filenames or guessing from profile names.

## What Changes

- Add manifest coverage metadata so asset entries can declare the celestial objects they cover.
- Add an ephemeris asset helper command that lists celestial objects only from selected profile assets that are present and valid locally.
- Report selected profile assets that are skipped because they are missing, invalid, or have no celestial object coverage metadata.
- Keep the command offline and deterministic; it does not download files or introspect kernel contents.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ephemeris-core`: Extend ephemeris asset manifest metadata and helper commands to report downloaded celestial object coverage for a selected profile.

## Impact

- Updates `crates/space-game-ephemeris` manifest types, parsing, validation, and tests.
- Updates `data/ephemeris/manifest.toml` with curated coverage metadata for existing assets where known.
- Adds an `objects` subcommand to the `ephemeris-assets` helper.
- Updates README or helper usage documentation for the new command.
