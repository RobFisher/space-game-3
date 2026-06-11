# Space ephemeris library design for a Rust TUI space adventure game

Status: design brief for OpenSpec explore mode  
Primary consumer: AI coding assistant implementing a Rust crate  
Working crate name: `space_ephemeris`  
Game context: real-time text adventure set in the Solar System, with real planets/moons plus fictional stations, ships and locations

---

## 1. Goal

Build a Rust library that can answer questions like:

```rust
let t = GameTime::from_utc("2097-04-12T18:30:00Z")?;
let mars = world.state("mars", t)?;
let europa = world.state("europa", t)?;
let station = world.state("port_lowell", t)?;
let distance = world.distance("player_ship", "deimos", t)?;
```

The library should provide a clean game-facing API over:

1. Real Solar System objects loaded from SPICE/JPL ephemeris kernels.
2. Game-authored objects such as stations, bases, ships, jump gates and scripted routes.
3. Optional fallback approximations where real kernels are unavailable.

The rest of the game should not care whether an object is real, fictional, fixed to a planet, orbiting, or following a sampled trajectory. Everything should be queryable as an object with a position, velocity and metadata at a timestamp.

---

## 2. Non-goals for the first implementation

Do not implement a full N-body simulator.

Do not attempt mission-grade spacecraft navigation.

Do not write custom SPICE kernels in v1.

Do not require internet access at game runtime.

Do not make the TUI depend directly on ANISE or SPICE concepts.

Do not require every moon, asteroid or comet in the first data pack.

---

## 3. Recommended external basis

Use a Rust-native SPICE-compatible library if possible.

Recommended first choice:

```toml
[dependencies]
anise = "0.10"
hifitime = "..."
serde = { version = "...", features = ["derive"] }
thiserror = "..."
glam = { version = "...", features = ["serde"] }
```

Notes for implementer:

- `anise` is a Rust-native replacement/rewrite of important NAIF SPICE functionality.
- Use it for reading SPK/BSP, PCK/TPC, FK and LSK-style data where supported.
- Hide ANISE behind this crate's API so it can be replaced later if needed.
- Use `hifitime` or ANISE's preferred time type for precise ephemeris time conversion, but expose a simple game-level time wrapper.

Fallback option:

- `rust-spice` / CSPICE wrapper, if ANISE lacks required behaviour.
- If CSPICE is used, guard global/shared state carefully and keep it behind a provider abstraction.

---

## 4. Data files and downloader

### 4.1 Kernel redistribution

NAIF-distributed kernels can generally be redistributed unchanged. Treat them as third-party data assets, not as code under this project's licence. Keep a third-party notice file and preserve filenames/comments.

Do not modify downloaded kernels. If a kernel ever needs modification, write a new file name and update attribution metadata.

### 4.2 Standard kernel sources

Primary source root:

```text
https://naif.jpl.nasa.gov/pub/naif/generic_kernels/
```

Useful subdirectories:

```text
spk/planets/       # planetary and lunar SPK/BSP files
spk/satellites/    # natural satellite SPK/BSP files
spk/asteroids/     # selected asteroid SPK/BSP files, but may be old/limited
lsk/               # leap seconds kernels
pck/               # planetary constants and orientation kernels
fk/                # frame kernels, if required
```

Suggested default data pack:

```text
latest_leapseconds.tls or naif0012.tls
pck00011.tpc, falling back to pck00010.tpc if needed
de442s.bsp if available, otherwise de440s.bsp
selected satellite kernels, initially:
  mar099.bsp or mar099s.bsp
  jup365.bsp or a newer compact Jupiter major-moons kernel
  sat*.bsp compact major-moons kernel chosen by manifest
  ura*.bsp compact major-moons kernel chosen by manifest
  nep*.bsp compact major-moons kernel chosen by manifest
  plu*.bsp compact Pluto-system kernel chosen by manifest
```

Important: do not hard-code every current filename in the code. Put filenames, URLs, checksums, sizes and expected coverage in a manifest.

### 4.3 Kernel manifest

Create a checked-in manifest, for example:

```toml
# data/kernels/standard.toml
schema_version = 1
profile = "standard"

[[kernels]]
id = "leapseconds"
kind = "lsk"
filename = "naif0012.tls"
url = "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/lsk/naif0012.tls"
sha256 = "TODO_FILL_FROM_TRUSTED_DOWNLOAD"
required = true

[[kernels]]
id = "planetary_constants"
kind = "pck"
filename = "pck00011.tpc"
url = "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/pck00011.tpc"
sha256 = "TODO_FILL_FROM_TRUSTED_DOWNLOAD"
required = true

[[kernels]]
id = "planets_short"
kind = "spk"
filename = "de442s.bsp"
url = "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de442s.bsp"
sha256 = "TODO_FILL_FROM_TRUSTED_DOWNLOAD"
required = true
covers = "Sun, planetary barycentres, Mercury, Venus, Earth, Moon"
preferred_range = "verify from kernel summary at build/fetch time"

[[kernels]]
id = "mars_satellites"
kind = "spk"
filename = "mar099.bsp"
url = "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/satellites/mar099.bsp"
sha256 = "TODO_FILL_FROM_TRUSTED_DOWNLOAD"
required = false
covers = "Mars, Phobos, Deimos"
```

The exact manifest should be generated/curated by a tool that can read the NAIF `aa_checksums.txt` and `aa_summaries.txt` files where available.

### 4.4 Downloader requirements

Provide a downloader as part of the repository. Prefer an explicit build/data step over implicit network access during normal `cargo build`.

Recommended shape:

```text
cargo xtask fetch-kernels --profile standard --dest .spacegame/kernels
cargo xtask verify-kernels --profile standard --dest .spacegame/kernels
cargo xtask update-kernel-manifest --profile standard
```

Also provide a Cargo alias:

```toml
[alias]
fetch-kernels = "run -p xtask -- fetch-kernels --profile standard"
verify-kernels = "run -p xtask -- verify-kernels --profile standard"
```

If the user explicitly wants build-time download, support it as an opt-in feature, not default behaviour:

```text
SPACEGAME_FETCH_KERNELS=1 cargo build --features fetch-kernels
```

Normal `cargo build` must be offline and reproducible if kernels are already present.

Downloader behaviour:

- Read a manifest.
- Create destination directory if needed.
- Download to a temporary file first.
- Verify checksum and size.
- Atomically rename into place only after verification.
- Skip files already present with matching checksum.
- Print third-party notice reminder after first download.
- Support `--offline` to fail fast if missing files.
- Support `--refresh` to re-download even if present.
- Support environment variable `SPACEGAME_KERNEL_DIR`.
- Write a local lock file recording fetched URLs, checksums and timestamps.

Do not perform network access during game runtime unless the user runs an explicit downloader command.

### 4.5 Third-party notices

Create:

```text
THIRD_PARTY_NOTICES.md
```

Include:

- Kernel filenames.
- Source URLs.
- Download date.
- SHA-256 checksums.
- Statement that kernels are unmodified.
- Link to NAIF rules page.
- Link to NAIF generic kernels root.

---

## 5. Core concepts

### 5.1 Object identity

Use string IDs for game-facing identity, not NAIF numeric IDs. Keep NAIF IDs as implementation details for real bodies.

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(String);
```

Examples:

```text
sun
earth
mars
phobos
europa
port_lowell
gateway_station
player_ship
```

### 5.2 Frames

Expose a small set of game-facing frames.

```rust
pub enum FrameId {
    SolarSystemBarycentricJ2000,
    ParentCenteredInertial(ObjectId),
    BodyFixed(ObjectId),
    Custom(String),
}
```

V1 should mainly use:

- Solar-system barycentric inertial frame for global positions.
- Parent-centred inertial frame for orbital calculations.
- Body-fixed frame for surface locations.

Internally, map these to ANISE/SPICE frames where possible.

### 5.3 State vector

Use km and km/s internally because SPICE commonly works with km and seconds.

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3Km {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3KmPerSec {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StateVector {
    pub position_km: Vec3Km,
    pub velocity_km_s: Vec3KmPerSec,
    pub frame: FrameId,
    pub epoch: GameTime,
    pub quality: EphemerisQuality,
}
```

### 5.4 Time

Expose a simple type.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GameTime {
    // internally backed by hifitime/ANISE epoch or a precise UTC representation
}

impl GameTime {
    pub fn from_utc_iso8601(s: &str) -> Result<Self, EphemerisError>;
    pub fn now_utc() -> Self;
}
```

Internally, convert to ephemeris time/TDB/ET as required by ANISE.

---

## 6. Object source model

Each object has an `EphemerisSource`.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EphemerisSource {
    SpiceBody {
        naif_id: i32,
        name: Option<String>,
        default_observer_naif_id: Option<i32>,
    },

    BodyFixed {
        parent: ObjectId,
        latitude_deg: f64,
        longitude_deg: f64,
        altitude_km: f64,
    },

    CircularOrbit {
        parent: ObjectId,
        radius_km: f64,
        period_seconds: f64,
        inclination_deg: f64,
        raan_deg: f64,
        phase_at_epoch_deg: f64,
        epoch: GameTime,
    },

    KeplerOrbit {
        parent: ObjectId,
        elements: OrbitalElements,
        epoch: GameTime,
    },

    LagrangePoint {
        primary: ObjectId,
        secondary: ObjectId,
        point: LagrangePoint,
        offset_km: Option<Vec3Km>,
    },

    SampledTrajectory {
        centre: ObjectId,
        frame: FrameId,
        samples: Vec<TrajectorySample>,
        interpolation: InterpolationMode,
    },

    FixedOffset {
        parent: ObjectId,
        offset_km: Vec3Km,
        frame: FrameId,
    },
}
```

### 6.1 Real bodies

Real bodies are backed by SPICE/ANISE kernels.

Example object registry entries:

```toml
[[objects]]
id = "earth"
name = "Earth"
kind = "planet"

[objects.source]
type = "spice_body"
naif_id = 399
name = "EARTH"

[[objects]]
id = "moon"
name = "Moon"
kind = "moon"

[objects.source]
type = "spice_body"
naif_id = 301
name = "MOON"
```

### 6.2 Surface bases

```toml
[[objects]]
id = "ulysses_base"
name = "Ulysses Base"
kind = "surface_base"

[objects.source]
type = "body_fixed"
parent = "mars"
latitude_deg = 12.4
longitude_deg = 88.2
altitude_km = 0.8
```

Resolution behaviour:

1. Resolve parent body state in inertial/global frame.
2. Convert body-fixed lat/lon/alt to a parent-relative Cartesian vector.
3. Rotate body-fixed vector into inertial frame using loaded PCK/frame data where available.
4. Add to parent state.

V1 fallback if rotation is not yet implemented:

- Allow `BodyFixed` objects to exist.
- Return an error `FrameTransformUnavailable` unless `allow_approximate_body_fixed = true` is configured.

### 6.3 Stations in simple circular orbits

```toml
[[objects]]
id = "ares_terminal"
name = "Ares Terminal"
kind = "station"

[objects.source]
type = "circular_orbit"
parent = "mars"
radius_km = 3850.0
period_seconds = 7200.0
inclination_deg = 30.0
raan_deg = 0.0
phase_at_epoch_deg = 15.0
epoch = "2097-01-01T00:00:00Z"
```

Circular orbit behaviour:

- Calculate angular displacement since epoch.
- Produce parent-relative position and velocity in parent-centred inertial frame.
- Transform/add parent global state.

This is enough for believable fictional stations.

### 6.4 Keplerian orbits

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrbitalElements {
    pub semi_major_axis_km: f64,
    pub eccentricity: f64,
    pub inclination_deg: f64,
    pub raan_deg: f64,
    pub argument_of_periapsis_deg: f64,
    pub mean_anomaly_at_epoch_deg: f64,
    pub gravitational_parameter_km3_s2: Option<f64>,
}
```

If `gravitational_parameter_km3_s2` is omitted, use the parent body's known GM from metadata if available.

V1 may postpone full Kepler support if circular orbits are enough.

### 6.5 Lagrange points

V1 approximation is acceptable.

```rust
pub enum LagrangePoint {
    L1,
    L2,
    L3,
    L4,
    L5,
}
```

Behaviour:

- Use primary and secondary states at time `t`.
- Compute approximate point in the rotating two-body frame.
- Convert to global frame.
- Apply optional offset.

Good enough for stations near Earth-Moon L1/L2, Sun-Earth L1/L2, Jupiter Trojan regions, etc.

### 6.6 Sampled trajectories

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrajectorySample {
    pub epoch: GameTime,
    pub position_km: Vec3Km,
    pub velocity_km_s: Vec3KmPerSec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InterpolationMode {
    Linear,
    CubicHermite,
}
```

Use for ships and scripted routes. V1 can start with linear interpolation.

---

## 7. Public API sketch

### 7.1 Library construction

```rust
pub struct SolarSystemBuilder {
    kernel_dir: Option<PathBuf>,
    manifest_path: Option<PathBuf>,
    object_registry_path: Option<PathBuf>,
    allow_approximate_fallbacks: bool,
}

impl SolarSystemBuilder {
    pub fn new() -> Self;
    pub fn kernel_dir(self, path: impl Into<PathBuf>) -> Self;
    pub fn kernel_manifest(self, path: impl Into<PathBuf>) -> Self;
    pub fn object_registry(self, path: impl Into<PathBuf>) -> Self;
    pub fn allow_approximate_fallbacks(self, enabled: bool) -> Self;
    pub fn build(self) -> Result<SolarSystem, EphemerisError>;
}
```

### 7.2 Querying states

```rust
pub struct SolarSystem {
    // owns an object registry and one or more providers
}

impl SolarSystem {
    pub fn state(
        &self,
        object: impl AsRef<str>,
        epoch: GameTime,
    ) -> Result<StateVector, EphemerisError>;

    pub fn state_relative_to(
        &self,
        target: impl AsRef<str>,
        observer: impl AsRef<str>,
        epoch: GameTime,
    ) -> Result<StateVector, EphemerisError>;

    pub fn position(
        &self,
        object: impl AsRef<str>,
        epoch: GameTime,
    ) -> Result<Vec3Km, EphemerisError>;

    pub fn distance(
        &self,
        a: impl AsRef<str>,
        b: impl AsRef<str>,
        epoch: GameTime,
    ) -> Result<f64, EphemerisError>;

    pub fn light_time_seconds(
        &self,
        a: impl AsRef<str>,
        b: impl AsRef<str>,
        epoch: GameTime,
    ) -> Result<f64, EphemerisError>;

    pub fn list_objects(&self) -> Vec<ObjectSummary>;

    pub fn object_metadata(
        &self,
        object: impl AsRef<str>,
    ) -> Result<ObjectMetadata, EphemerisError>;
}
```

### 7.3 Trait abstraction

```rust
pub trait EphemerisProvider: Send + Sync {
    fn state(
        &self,
        object: &ObjectDefinition,
        epoch: GameTime,
        frame: FrameId,
        registry: &ObjectRegistry,
    ) -> Result<StateVector, EphemerisError>;
}
```

Implementations:

```rust
pub struct SpiceProvider { /* wraps ANISE Almanac */ }
pub struct GameObjectProvider { /* circular, kepler, fixed, sampled */ }
pub struct CompositeProvider { /* dispatches by EphemerisSource */ }
```

### 7.4 Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum EphemerisError {
    #[error("unknown object: {0}")]
    UnknownObject(String),

    #[error("kernel not found: {0}")]
    KernelNotFound(String),

    #[error("object {object} is outside available ephemeris coverage at {epoch:?}")]
    OutOfCoverage { object: String, epoch: GameTime },

    #[error("frame transform unavailable: {0}")]
    FrameTransformUnavailable(String),

    #[error("invalid object definition: {0}")]
    InvalidObjectDefinition(String),

    #[error("cyclic object dependency involving: {0}")]
    CyclicDependency(String),

    #[error("download failed: {0}")]
    DownloadFailed(String),

    #[error("checksum mismatch for {filename}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        filename: String,
        expected: String,
        actual: String,
    },

    #[error("backend error: {0}")]
    Backend(String),
}
```

---

## 8. Internal resolution algorithm

### 8.1 Global state resolution

`SolarSystem::state(object, t)` should return state in `SolarSystemBarycentricJ2000` unless otherwise configured.

Pseudo-code:

```rust
fn resolve_global_state(object_id, t, visited) -> Result<StateVector> {
    if visited.contains(object_id) {
        return Err(CyclicDependency(object_id));
    }

    let obj = registry.get(object_id)?;

    match obj.source {
        SpiceBody { .. } => spice_provider.state(obj, t, SolarSystemBarycentricJ2000),

        BodyFixed { parent, .. } => {
            let parent_state = resolve_global_state(parent, t, visited)?;
            let local = body_fixed_to_parent_inertial(obj, t)?;
            Ok(parent_state + local)
        }

        CircularOrbit { parent, .. } => {
            let parent_state = resolve_global_state(parent, t, visited)?;
            let local = circular_orbit_state(obj, t)?;
            Ok(parent_state + local)
        }

        KeplerOrbit { parent, .. } => {
            let parent_state = resolve_global_state(parent, t, visited)?;
            let local = kepler_orbit_state(obj, t)?;
            Ok(parent_state + local)
        }

        LagrangePoint { primary, secondary, .. } => {
            let primary_state = resolve_global_state(primary, t, visited)?;
            let secondary_state = resolve_global_state(secondary, t, visited)?;
            approximate_lagrange_state(primary_state, secondary_state, obj, t)
        }

        SampledTrajectory { centre, .. } => {
            let centre_state = resolve_global_state(centre, t, visited)?;
            let local = interpolate_trajectory(obj, t)?;
            Ok(centre_state + local)
        }

        FixedOffset { parent, .. } => {
            let parent_state = resolve_global_state(parent, t, visited)?;
            Ok(parent_state + fixed_offset_state(obj, t)?)
        }
    }
}
```

### 8.2 Caching

Because the game is real-time, many objects may be queried repeatedly for the same tick.

Implement optional per-tick cache:

```rust
pub struct StateCacheKey {
    object: ObjectId,
    epoch_quantized_ns: i128,
    frame: FrameId,
}
```

V1 can use a simple `HashMap` behind `SolarSystemTickContext`:

```rust
pub struct SolarSystemTickContext<'a> {
    world: &'a SolarSystem,
    epoch: GameTime,
    cache: HashMap<ObjectId, StateVector>,
}

impl<'a> SolarSystemTickContext<'a> {
    pub fn state(&mut self, object: impl AsRef<str>) -> Result<StateVector, EphemerisError>;
    pub fn distance(&mut self, a: impl AsRef<str>, b: impl AsRef<str>) -> Result<f64, EphemerisError>;
}
```

---

## 9. Metadata model

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectDefinition {
    pub id: ObjectId,
    pub name: String,
    pub kind: ObjectKind,
    pub source: EphemerisSource,
    pub physical: Option<PhysicalProperties>,
    pub gameplay: Option<GameplayMetadata>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectKind {
    Star,
    Planet,
    DwarfPlanet,
    Moon,
    Asteroid,
    Comet,
    Station,
    SurfaceBase,
    Ship,
    JumpGate,
    Region,
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhysicalProperties {
    pub mean_radius_km: Option<f64>,
    pub gravitational_parameter_km3_s2: Option<f64>,
    pub rotation_period_seconds: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameplayMetadata {
    pub description: Option<String>,
    pub faction: Option<String>,
    pub tags: Vec<String>,
    pub discoverable: bool,
}
```

Example registry:

```toml
[[objects]]
id = "port_lowell"
name = "Port Lowell"
kind = "station"

[objects.source]
type = "circular_orbit"
parent = "mars"
radius_km = 3850.0
period_seconds = 7200.0
inclination_deg = 30.0
raan_deg = 0.0
phase_at_epoch_deg = 15.0
epoch = "2097-01-01T00:00:00Z"

[objects.gameplay]
description = "A busy transfer station above Mars."
faction = "lowell_authority"
tags = ["trade", "shipyard", "mars"]
discoverable = true
```

---

## 10. Validation strategy

### 10.1 Unit tests

Test pure game object maths independently from SPICE:

- Circular orbit returns same position after one period.
- Circular orbit velocity is perpendicular to radius for zero inclination.
- Distance between identical object is zero.
- Recursive parent-child resolution adds vectors correctly.
- Cyclic dependencies are detected.
- Sampled trajectory interpolation works.
- Out-of-range sampled trajectories fail clearly.

### 10.2 Kernel tests

With kernels present:

- Load standard manifest.
- Query Sun, Earth, Moon, Mars at a known timestamp.
- Verify that state vectors are finite and sane.
- Verify known target IDs are present.
- Verify out-of-coverage timestamps return `OutOfCoverage`.

### 10.3 Cross-check tests

Add optional ignored tests that compare selected outputs against JPL Horizons or known generated reference vectors.

```rust
#[test]
#[ignore = "requires network or checked-in reference data"]
fn earth_position_matches_horizons_reference() { ... }
```

Prefer checked-in small reference JSON fixtures over live network tests.

---

## 11. CLI tools

Create a small CLI binary for debugging and data checks.

```text
space-ephem list-objects
space-ephem state mars --at 2097-04-12T18:30:00Z
space-ephem distance earth mars --at 2097-04-12T18:30:00Z
space-ephem coverage
space-ephem kernels verify
space-ephem kernels fetch --profile standard
```

This CLI is useful for both humans and AI agents during development.

Example output:

```text
Object: mars
Epoch: 2097-04-12T18:30:00Z
Frame: SolarSystemBarycentricJ2000
Position km: [-1.234e8, 2.345e8, 1.234e7]
Velocity km/s: [-21.4, -10.2, 0.3]
Quality: real_kernel
```

---

## 12. Suggested implementation phases

### Phase 1: API and fictional object maths

Deliver:

- Object registry loader from TOML.
- `GameTime`, `StateVector`, `ObjectId`.
- `SolarSystemBuilder`.
- Circular orbit support.
- Fixed offset support.
- Sampled trajectory support with linear interpolation.
- Distance and light-time helpers.
- Tests for all pure maths.

No SPICE required in this phase.

### Phase 2: Kernel manifest and downloader

Deliver:

- Manifest parser.
- `xtask fetch-kernels`.
- `xtask verify-kernels`.
- SHA-256 validation.
- Third-party notices generation.
- Offline mode.

### Phase 3: ANISE-backed real bodies

Deliver:

- `SpiceProvider` wrapping ANISE.
- Load kernels from manifest.
- Resolve `SpiceBody` objects.
- Query planets and Moon.
- Coverage errors.
- CLI `state` and `coverage` commands.

### Phase 4: Moons and body-fixed locations

Deliver:

- Satellite kernel profiles.
- Major moons registry.
- PCK-backed body-fixed transforms if available.
- Surface base support.

### Phase 5: Better gameplay objects

Deliver:

- Keplerian orbit support.
- Approximate Lagrange points.
- Tick cache.
- Optional custom object packs.

### Phase 6: Horizons/small-body support

Deliver:

- Tool to request/generate SPK for selected asteroids/comets via JPL Horizons.
- Cache generated small-body SPKs.
- Register Ceres, Vesta, Pallas, Hygiea, etc.

This should remain explicit and optional.

---

## 13. Acceptance criteria for v1

The crate is successful when the following works:

```rust
let world = SolarSystemBuilder::new()
    .kernel_dir(".spacegame/kernels")
    .kernel_manifest("data/kernels/standard.toml")
    .object_registry("data/objects/core.toml")
    .build()?;

let t = GameTime::from_utc_iso8601("2097-04-12T18:30:00Z")?;

let earth = world.state("earth", t)?;
let mars = world.state("mars", t)?;
let port_lowell = world.state("port_lowell", t)?;
let d = world.distance("earth", "mars", t)?;
let delay = world.light_time_seconds("earth", "mars", t)?;
```

And:

- Normal game runtime performs no network access.
- Kernel files can be fetched and verified by command.
- Missing kernels produce helpful errors.
- Fictional stations can be queried even without real kernels if their parent objects are mocked or approximated.
- Real bodies and fictional bodies share one object registry and one query API.

---

## 14. Important design choices to preserve

1. Keep the game-facing API independent of ANISE/SPICE.
2. Use string `ObjectId`s externally, NAIF IDs internally.
3. Treat ephemeris kernels as data assets with manifests and checksums.
4. Prefer explicit `xtask fetch-kernels` over hidden network access in `build.rs`.
5. Support custom game objects as first-class objects, not as SPICE kernels in v1.
6. Use km and seconds internally.
7. Return quality metadata so UI/gameplay can distinguish real, approximate and fictional positions.
8. Make all time and frame conversion failures explicit.
9. Add a CLI early to make the library debuggable.
10. Keep phases small enough for an AI coding agent to implement safely.

---

## 15. Open questions for explore mode

Ask the AI coding agent to investigate:

1. Whether ANISE currently exposes all required frame and body-fixed transform APIs for this project.
2. Whether ANISE can directly load `naif0012.tls`, `pck00011.tpc`, `de442s.bsp`, and selected satellite kernels without conversion.
3. Whether `de442s.bsp` should be the default planetary kernel, or whether `de440s.bsp` is currently better supported/documented by the Rust ecosystem.
4. Which compact satellite kernels give the best size/coverage trade-off for Mars, Jupiter, Saturn, Uranus, Neptune and Pluto major moons.
5. Whether `glam`, `nalgebra`, or a tiny custom `Vec3` type is best for this library.
6. How to expose time types ergonomically without leaking too much `hifitime` complexity.
7. Whether body-fixed locations should be implemented in v1 or postponed behind a clear error.
8. Whether the downloader should live in `xtask`, a separate `space-ephem-data` CLI, or both.
9. Whether to use `ureq`, `reqwest`, or `curl`-style external commands for downloading.
10. Whether to support a `build.rs` opt-in feature for users who really want `cargo build` to fetch kernels.

---

## 16. Suggested first OpenSpec instruction

Use this as the first explore-mode instruction:

```text
Explore this design and propose a Rust crate structure for `space_ephemeris`.
Do not implement yet.
Focus on module boundaries, public API, data manifest format, and how to wrap ANISE without leaking it into the game-facing API.
Also investigate whether the named NAIF kernels can be loaded by ANISE directly and whether an explicit `xtask fetch-kernels` downloader is preferable to a `build.rs` downloader.
Keep the first implementation small: object registry, fictional circular orbits, manifest parsing, and a stubbed SPICE provider are enough for the first proposal.
```

