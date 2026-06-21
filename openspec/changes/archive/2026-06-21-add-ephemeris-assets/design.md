## Context

The ephemeris crate already parses a simple kernel manifest and exposes it through the game-facing ephemeris API without performing network access. Real solar-system data files still need a repeatable open source workflow: developers should be able to see which files are needed, fetch them explicitly, verify them, and keep the large downloaded files outside git.

The server currently uses fictional/demo registry data and the SPICE provider remains a stub. This change prepares the asset management layer for real kernels without requiring the real SPICE-backed state resolver to be implemented in the same change.

## Goals / Non-Goals

**Goals:**

- Define a checked-in, profile-based ephemeris asset manifest format.
- Use repo-root `data/ephemeris/` as the default asset root.
- Support `SPACE_GAME_EPHEMERIS_DATA_DIR` as an override for the asset root.
- Add explicit `list`, `verify`, and `fetch` helper commands for manifest profiles.
- Keep downloads out of normal compilation, tests, and default library use.
- Ensure tests use fixture manifests and local files rather than internet access.

**Non-Goals:**

- Implement full SPICE/ANISE state resolution.
- Guarantee the correctness of every real upstream URL or checksum forever.
- Vendor large ephemeris assets into git.
- Require CI to download large real kernel files.

## Decisions

Use a profile-based manifest keyed by asset id. The manifest will use `version = 1`, `[profiles.<name>]` tables that list asset ids, and `[assets.<id>]` tables that describe each asset. This keeps profile selection explicit and avoids duplicating asset metadata across profiles. The existing linear `kernels[]` structure is too limited because it has only one profile and does not represent reusable asset sets.

Resolve asset files relative to an asset root, not relative to the manifest file. The default root is `<repo>/data/ephemeris`; if `SPACE_GAME_EPHEMERIS_DATA_DIR` is set, that directory becomes the asset root. Manifest `local_path` values remain relative to the asset root. This lets developers keep large data on another volume without changing the checked-in manifest.

Keep downloaded files ignored but keep the manifest tracked. The repository will track `data/ephemeris/manifest.toml` and ignore `data/ephemeris/kernels/`. This makes setup reproducible while avoiding large binary files in source control.

Add an explicit helper binary in `space-game-ephemeris`. The command shape should be close to:

```sh
cargo run -p space-game-ephemeris --bin ephemeris-assets -- list --profile minimal
cargo run -p space-game-ephemeris --bin ephemeris-assets -- verify --profile minimal
cargo run -p space-game-ephemeris --bin ephemeris-assets -- fetch --profile minimal
```

A crate-local binary is smaller than adding an `xtask` crate now and keeps the manifest model near the commands that use it. A future `xtask` can wrap or call the same library code if the workspace accumulates more maintenance commands.

Treat `sha256` and `size_bytes` as verification fields and `approx_size` as human-readable documentation. `verify` MUST enforce `sha256` when present and MUST enforce `size_bytes` when present. `approx_size` MUST NOT be used as a correctness check.

Validate paths defensively. `local_path` values must be relative, normalized, and must not contain parent-directory traversal. The helper should create parent directories for downloads, write to a temporary file under the asset root, verify it, and then move it into place.

## Risks / Trade-offs

- Real upstream URLs, filenames, or checksums can change -> keep the manifest easy to update and ensure failures identify the asset id and URL.
- Adding HTTP and CLI dependencies can bloat the core crate -> keep network behavior in the helper binary and shared non-network manifest/verification code in the library.
- Missing checksums reduce verification strength -> allow manifests to parse with absent checksums, but clearly report weaker verification and enforce checksums when available.
- Environment overrides can make support harder -> print the resolved asset root in helper output and errors.
- Large downloads can make CI brittle -> CI tests must use fixture assets; real download checks should remain opt-in.
