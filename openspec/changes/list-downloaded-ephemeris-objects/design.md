## Context

The ephemeris asset manifest currently describes profile membership and downloadable files. The helper binary can list profile assets, verify local files, and fetch files explicitly. Developers can therefore answer "which files are in this profile?" and "is this profile complete?", but not "which celestial objects do my downloaded profile files make available?"

The game object registry is separate from the asset manifest. Registry objects describe game-facing identities and sources, while manifest assets describe downloadable kernels and data files. The current SPICE-backed resolution path is intentionally future-facing, so this change should not depend on introspecting kernel contents.

## Goals / Non-Goals

**Goals:**

- Add explicit, curated celestial object coverage metadata to manifest assets.
- Add a helper command that lists objects only from selected profile assets that are valid on disk.
- Preserve offline tests and avoid implicit downloads.
- Make skipped assets visible so partial profiles are easy to understand.

**Non-Goals:**

- Do not parse SPK/PCK/ANISE files to discover contents.
- Do not make the game server or TUI use this coverage list as the authoritative object registry.
- Do not implement SPICE-backed object state resolution.
- Do not require every possible object covered by a kernel to be listed before the command is useful.

## Decisions

1. **Use curated manifest coverage metadata.**

   Add a `covers` list on asset entries containing celestial object id, display name, kind, optional NAIF id, and optional notes. This keeps the feature deterministic, reviewable, and testable without external kernel tooling.

   Alternative considered: inspect downloaded kernels directly. That is more complete in theory, but it introduces provider/tooling coupling before the SPICE backend is ready and would make offline fixture tests more complex.

2. **List only objects from valid local assets.**

   The new helper command answers the inventory question "what do I have downloaded?", so it should verify each selected profile asset and emit coverage only for assets that are present and pass available size/checksum checks.

   Alternative considered: list all profile coverage and mark missing objects. That is useful for planning downloads, but it blurs downloaded availability with theoretical profile membership. Existing `list` and `verify` already cover profile expectations.

3. **Skip bad or unhelpful assets without aborting the whole object listing.**

   If a selected asset is missing, invalid, or has no coverage metadata, the command should omit its objects and include a skipped-assets summary with the reason. This makes partial profile states useful instead of forcing developers to repair every selected asset before seeing any inventory.

   Alternative considered: reuse whole-profile verification and fail on the first required missing asset. That matches `verify`, but it does not match the inventory use case.

4. **Deduplicate by object id.**

   Multiple assets may cover the same body. The command should list each object once, with enough source information to explain which valid downloaded asset contributed it. If duplicates differ in metadata, validation should reject conflicting duplicate coverage within a single manifest where practical, or the command should choose a deterministic first-by-profile order and keep output stable.

## Risks / Trade-offs

- Curated coverage can become stale or incomplete -> Keep metadata close to the manifest entries and cover it with manifest parsing/validation tests.
- Skipping invalid required assets could hide profile incompleteness -> The command must report skipped assets and `verify` remains the strict completeness check.
- Duplicate coverage can confuse output -> Deduplicate deterministically and test duplicate scenarios.
- Object kinds may drift from the game registry's `ObjectKind` vocabulary -> Reuse the existing object kind model where practical so manifest coverage and registry terminology stay aligned.
