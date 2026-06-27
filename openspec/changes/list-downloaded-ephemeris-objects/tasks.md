## 1. Manifest Coverage Model

- [x] 1.1 Add a covered celestial object type to the ephemeris manifest model with id, display name, object kind, optional NAIF id, and optional notes.
- [x] 1.2 Parse optional `covers` metadata on asset entries and preserve it through manifest loading.
- [x] 1.3 Validate coverage entries for non-empty ids/names, supported kinds, and conflicting duplicate object ids.
- [x] 1.4 Add fixture tests for valid coverage metadata and invalid coverage metadata.

## 2. Downloaded Object Listing

- [x] 2.1 Add per-asset verification support suitable for continuing after missing or invalid selected assets.
- [x] 2.2 Add an `objects` subcommand to `ephemeris-assets` with `--profile`, `--manifest`, and `--asset-root` support.
- [x] 2.3 List only covered objects from selected profile assets that are present and valid locally.
- [x] 2.4 Deduplicate listed objects by id in deterministic output order while preserving source asset information.
- [x] 2.5 Report skipped selected assets with reasons for missing files, invalid files, or absent coverage metadata.

## 3. Data, Docs, and Verification

- [x] 3.1 Add curated coverage metadata to `data/ephemeris/manifest.toml` for existing assets where known.
- [x] 3.2 Add helper command tests using local fixture assets rather than network downloads.
- [x] 3.3 Update README usage examples for listing downloaded celestial objects.
- [x] 3.4 Run relevant Rust tests for `space-game-ephemeris`.
- [x] 3.5 Run `openspec validate --all`.
