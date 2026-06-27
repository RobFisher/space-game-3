use space_game_ephemeris::{
    default_asset_root, downloaded_profile_objects, fetch_profile_assets, resolve_asset_path,
    verify_profile_assets, AssetVerificationStatus, EphemerisAssetManifest, EphemerisError,
    EphemerisQuality, FrameId, GameTime, ObjectKind, ObjectRegistry, SkippedAssetReason,
    SolarSystemBuilder, StateVector, Vec3Km, Vec3KmPerSec,
};
use std::fs;

fn epoch() -> GameTime {
    GameTime::from_utc_iso8601("2097-01-01T00:00:00Z").unwrap()
}

fn fixture_registry() -> ObjectRegistry {
    ObjectRegistry::from_toml_str(
        r#"
[[objects]]
id = "origin"
name = "Origin"
kind = "region"

[objects.source]
type = "static_state"
position_km = { x = 10.0, y = 20.0, z = 30.0 }
velocity_km_s = { x = 1.0, y = 2.0, z = 3.0 }
frame = { type = "solar_system_barycentric_j2000" }

[[objects]]
id = "offset"
name = "Offset"
kind = "station"

[objects.source]
type = "fixed_offset"
parent = "origin"
offset_km = { x = 4.0, y = 5.0, z = 6.0 }
frame = { type = "parent_centered_inertial", value = "origin" }

[[objects]]
id = "orbiter"
name = "Orbiter"
kind = "station"

[objects.source]
type = "circular_orbit"
parent = "origin"
radius_km = 100.0
period_seconds = 10.0
inclination_deg = 0.0
raan_deg = 0.0
phase_at_epoch_deg = 0.0
epoch = "2097-01-01T00:00:00Z"

[[objects]]
id = "sampled"
name = "Sampled"
kind = "ship"

[objects.source]
type = "sampled_trajectory"
centre = "origin"
frame = { type = "parent_centered_inertial", value = "origin" }
interpolation = "linear"

[[objects.source.samples]]
epoch = "2097-01-01T00:00:00Z"
position_km = { x = 0.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 1.0, z = 0.0 }

[[objects.source.samples]]
epoch = "2097-01-01T00:00:10Z"
position_km = { x = 10.0, y = 20.0, z = 30.0 }
velocity_km_s = { x = 2.0, y = 3.0, z = 4.0 }
"#,
    )
    .unwrap()
}

fn fixture_world() -> space_game_ephemeris::SolarSystem {
    SolarSystemBuilder::new()
        .object_registry_data(fixture_registry())
        .build()
        .unwrap()
}

#[test]
fn game_time_parses_orders_and_adds_seconds() {
    let t = epoch();
    let later = t.add_seconds(2.5);

    assert_eq!(later.seconds_since(&t), 2.5);
    assert!(later > t);
    assert_eq!(t.to_string(), "2097-01-01T00:00:00Z");
}

#[test]
fn vector_math_and_state_combination_work() {
    let a = Vec3Km::new(1.0, 2.0, 3.0);
    let b = Vec3Km::new(4.0, 6.0, 3.0);
    assert_eq!(a.distance(b), 5.0);
    assert!(a.is_finite());

    let parent = StateVector::new(
        a,
        Vec3KmPerSec::new(1.0, 0.0, 0.0),
        FrameId::SolarSystemBarycentricJ2000,
        epoch(),
        EphemerisQuality::Fictional,
    );
    let local = StateVector::new(
        Vec3Km::new(10.0, 0.0, 0.0),
        Vec3KmPerSec::new(0.0, 1.0, 0.0),
        FrameId::ParentCenteredInertial("origin".into()),
        epoch(),
        EphemerisQuality::Fictional,
    );

    let combined = StateVector::combine_parent_local(&parent, &local);
    assert_eq!(combined.position_km, Vec3Km::new(11.0, 2.0, 3.0));
    assert_eq!(combined.velocity_km_s, Vec3KmPerSec::new(1.0, 1.0, 0.0));
}

#[test]
fn registry_loads_metadata_and_rejects_duplicates() {
    let registry = ObjectRegistry::from_toml_str(
        r#"
[[objects]]
id = "origin"
name = "Origin"
kind = "region"

[objects.source]
type = "static_state"
position_km = { x = 0.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }

[objects.gameplay]
description = "A fixture origin."
faction = "testers"
tags = ["fixture"]
discoverable = false
"#,
    )
    .unwrap();

    let metadata = registry.get("origin").unwrap();
    assert_eq!(metadata.kind, ObjectKind::Region);
    assert_eq!(
        metadata.gameplay.as_ref().unwrap().description.as_deref(),
        Some("A fixture origin.")
    );
    assert!(!metadata.gameplay.as_ref().unwrap().discoverable);

    let duplicate = ObjectRegistry::from_toml_str(
        r#"
[[objects]]
id = "origin"
name = "Origin"
kind = "region"
[objects.source]
type = "static_state"
position_km = { x = 0.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }

[[objects]]
id = "origin"
name = "Duplicate"
kind = "region"
[objects.source]
type = "static_state"
position_km = { x = 0.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }
"#,
    );
    assert!(matches!(
        duplicate,
        Err(EphemerisError::InvalidObjectDefinition(_))
    ));
}

#[test]
fn registry_rejects_invalid_definitions() {
    let invalid_orbit = ObjectRegistry::from_toml_str(
        r#"
[[objects]]
id = "origin"
name = "Origin"
kind = "region"
[objects.source]
type = "static_state"
position_km = { x = 0.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }

[[objects]]
id = "bad"
name = "Bad"
kind = "station"
[objects.source]
type = "circular_orbit"
parent = "origin"
radius_km = 0.0
period_seconds = 10.0
inclination_deg = 0.0
raan_deg = 0.0
phase_at_epoch_deg = 0.0
epoch = "2097-01-01T00:00:00Z"
"#,
    );

    assert!(matches!(
        invalid_orbit,
        Err(EphemerisError::InvalidObjectDefinition(_))
    ));
}

#[test]
fn fixed_offset_resolves_parent_state() {
    let world = fixture_world();
    let state = world.state("offset", epoch()).unwrap();

    assert_eq!(state.position_km, Vec3Km::new(14.0, 25.0, 36.0));
    assert_eq!(state.velocity_km_s, Vec3KmPerSec::new(1.0, 2.0, 3.0));
}

#[test]
fn dependency_cycles_are_reported() {
    let registry = ObjectRegistry::from_toml_str(
        r#"
[[objects]]
id = "a"
name = "A"
kind = "station"
[objects.source]
type = "fixed_offset"
parent = "b"
offset_km = { x = 0.0, y = 0.0, z = 0.0 }
frame = { type = "parent_centered_inertial", value = "b" }

[[objects]]
id = "b"
name = "B"
kind = "station"
[objects.source]
type = "fixed_offset"
parent = "a"
offset_km = { x = 0.0, y = 0.0, z = 0.0 }
frame = { type = "parent_centered_inertial", value = "a" }
"#,
    )
    .unwrap();
    let world = SolarSystemBuilder::new()
        .object_registry_data(registry)
        .build()
        .unwrap();

    assert!(matches!(
        world.state("a", epoch()),
        Err(EphemerisError::CyclicDependency(_))
    ));
}

#[test]
fn circular_orbit_repeats_and_has_tangential_velocity() {
    let world = fixture_world();
    let at_epoch = world.state("orbiter", epoch()).unwrap();
    let one_period = world.state("orbiter", epoch().add_seconds(10.0)).unwrap();

    assert!((at_epoch.position_km.distance(one_period.position_km)) < 1e-9);

    let parent = world.state("origin", epoch()).unwrap();
    let radius = at_epoch.position_km - parent.position_km;
    let velocity = at_epoch.velocity_km_s - parent.velocity_km_s;
    assert!(velocity.dot_position(radius).abs() < 1e-9);
}

#[test]
fn sampled_trajectory_interpolates_exact_samples_and_range_errors() {
    let world = fixture_world();
    let exact = world.state("sampled", epoch()).unwrap();
    assert_eq!(exact.position_km, Vec3Km::new(10.0, 20.0, 30.0));

    let midway = world.state("sampled", epoch().add_seconds(5.0)).unwrap();
    assert_eq!(midway.position_km, Vec3Km::new(15.0, 30.0, 45.0));
    assert_eq!(midway.velocity_km_s, Vec3KmPerSec::new(2.0, 4.0, 5.0));

    assert!(matches!(
        world.state("sampled", epoch().add_seconds(11.0)),
        Err(EphemerisError::OutOfCoverage { .. })
    ));
}

#[test]
fn public_api_reports_unknown_objects_distance_and_light_time() {
    let world = fixture_world();

    assert!(matches!(
        world.state("missing", epoch()),
        Err(EphemerisError::UnknownObject(id)) if id == "missing"
    ));
    assert_eq!(
        world.distance("origin", "offset", epoch()).unwrap(),
        (77.0_f64).sqrt()
    );
    let light_time = world
        .light_time_seconds("origin", "offset", epoch())
        .unwrap();
    assert!((light_time - (77.0_f64).sqrt() / 299_792.458).abs() < 1e-12);
    assert_eq!(world.list_objects().len(), 4);
    assert_eq!(world.object_metadata("origin").unwrap().name, "Origin");
}

#[test]
fn unsupported_sources_return_clear_errors() {
    let spice_registry = ObjectRegistry::from_toml_str(
        r#"
[[objects]]
id = "earth"
name = "Earth"
kind = "planet"
[objects.source]
type = "spice_body"
naif_id = 399
name = "EARTH"
"#,
    )
    .unwrap();
    let spice_world = SolarSystemBuilder::new()
        .object_registry_data(spice_registry)
        .build()
        .unwrap();
    assert!(matches!(
        spice_world.state("earth", epoch()),
        Err(EphemerisError::Backend(_))
    ));

    let body_fixed_registry = ObjectRegistry::from_toml_str(
        r#"
[[objects]]
id = "mars"
name = "Mars"
kind = "planet"
[objects.source]
type = "static_state"
position_km = { x = 0.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }

[[objects]]
id = "base"
name = "Base"
kind = "surface_base"
[objects.source]
type = "body_fixed"
parent = "mars"
latitude_deg = 12.4
longitude_deg = 88.2
altitude_km = 0.8
"#,
    )
    .unwrap();
    let body_fixed_world = SolarSystemBuilder::new()
        .object_registry_data(body_fixed_registry)
        .build()
        .unwrap();
    assert!(matches!(
        body_fixed_world.state("base", epoch()),
        Err(EphemerisError::FrameTransformUnavailable(_))
    ));
}

fn fixture_asset_manifest() -> EphemerisAssetManifest {
    EphemerisAssetManifest::from_toml_str(
        r#"
version = 1

[profiles.minimal]
description = "Fixture profile"
assets = ["fixture"]

[profiles.optional]
description = "Optional profile"
assets = ["optional"]

[profiles.mixed]
description = "Mixed profile"
assets = ["fixture", "optional", "metadata"]

[assets.fixture]
kind = "spk"
filename = "fixture.bsp"
source = "fixture"
url = "file:///tmp/fixture.bsp"
local_path = "kernels/fixture.bsp"
sha256 = "f0dad327e22e8cddc2e8057cf16d9b16ea6e36e87d31f46ee4d5943c69609c4f"
size_bytes = 14
required = true
description = "Fixture asset"

[[assets.fixture.covers]]
id = "earth"
name = "Earth"
kind = "planet"
naif_id = 399
notes = "Fixture coverage"

[[assets.fixture.covers]]
id = "moon"
name = "Moon"
kind = "moon"
naif_id = 301

[assets.optional]
kind = "spk"
filename = "optional.bsp"
source = "fixture"
url = "file:///tmp/optional.bsp"
local_path = "kernels/optional.bsp"
required = false

[[assets.optional.covers]]
id = "phobos"
name = "Phobos"
kind = "moon"
naif_id = 401

[assets.metadata]
kind = "anise-pca"
filename = "metadata.pca"
source = "fixture"
url = "file:///tmp/metadata.pca"
local_path = "kernels/metadata.pca"
required = false
"#,
    )
    .unwrap()
}

#[test]
fn ephemeris_asset_manifest_parses_selects_profiles_and_rejects_invalid_data() {
    let manifest = fixture_asset_manifest();

    let selected = manifest.profile_assets("minimal").unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].id, "fixture");
    assert!(selected[0].asset.required);
    assert_eq!(selected[0].asset.covers.len(), 2);
    assert_eq!(selected[0].asset.covers[0].id.as_str(), "earth");
    assert_eq!(selected[0].asset.covers[0].kind, ObjectKind::Planet);
    assert_eq!(selected[0].asset.covers[0].naif_id, Some(399));

    assert!(matches!(
        manifest.profile_assets("missing"),
        Err(EphemerisError::InvalidObjectDefinition(_))
    ));

    let invalid_reference = EphemerisAssetManifest::from_toml_str(
        r#"
version = 1
[profiles.bad]
description = "Bad"
assets = ["missing"]

[assets.fixture]
kind = "spk"
filename = "fixture.bsp"
source = "fixture"
url = "file:///tmp/fixture.bsp"
local_path = "kernels/fixture.bsp"
"#,
    );
    assert!(matches!(
        invalid_reference,
        Err(EphemerisError::InvalidObjectDefinition(_))
    ));

    let unsafe_path = EphemerisAssetManifest::from_toml_str(
        r#"
version = 1
[profiles.bad]
description = "Bad"
assets = ["fixture"]

[assets.fixture]
kind = "spk"
filename = "naif0012.tls"
source = "fixture"
url = "file:///tmp/fixture.bsp"
local_path = "../escape.bsp"
required = true
"#,
    );
    assert!(matches!(
        unsafe_path,
        Err(EphemerisError::InvalidObjectDefinition(_))
    ));

    let duplicate_profile_asset = EphemerisAssetManifest::from_toml_str(
        r#"
version = 1
[profiles.bad]
description = "Bad"
assets = ["fixture", "fixture"]

[assets.fixture]
kind = "spk"
filename = "fixture.bsp"
source = "fixture"
url = "file:///tmp/fixture.bsp"
local_path = "kernels/fixture.bsp"
"#,
    );
    assert!(matches!(
        duplicate_profile_asset,
        Err(EphemerisError::InvalidObjectDefinition(_))
    ));

    let invalid_coverage = EphemerisAssetManifest::from_toml_str(
        r#"
version = 1
[profiles.bad]
description = "Bad"
assets = ["fixture"]

[assets.fixture]
kind = "spk"
filename = "fixture.bsp"
source = "fixture"
url = "file:///tmp/fixture.bsp"
local_path = "kernels/fixture.bsp"

[[assets.fixture.covers]]
id = ""
name = "No Id"
kind = "planet"
"#,
    );
    assert!(matches!(
        invalid_coverage,
        Err(EphemerisError::InvalidObjectDefinition(_))
    ));

    let conflicting_coverage = EphemerisAssetManifest::from_toml_str(
        r#"
version = 1
[profiles.bad]
description = "Bad"
assets = ["first", "second"]

[assets.first]
kind = "spk"
filename = "first.bsp"
source = "fixture"
url = "file:///tmp/first.bsp"
local_path = "kernels/first.bsp"

[[assets.first.covers]]
id = "earth"
name = "Earth"
kind = "planet"

[assets.second]
kind = "spk"
filename = "second.bsp"
source = "fixture"
url = "file:///tmp/second.bsp"
local_path = "kernels/second.bsp"

[[assets.second.covers]]
id = "earth"
name = "Terra"
kind = "planet"
"#,
    );
    assert!(matches!(
        conflicting_coverage,
        Err(EphemerisError::InvalidObjectDefinition(_))
    ));
}

#[test]
fn asset_paths_and_offline_verification_report_clear_results() {
    let manifest = fixture_asset_manifest();
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let asset = &manifest.profile_assets("minimal").unwrap()[0];
    let path = resolve_asset_path(root, asset.asset);

    assert!(path.ends_with("kernels/fixture.bsp"));
    assert!(default_asset_root().ends_with("data/ephemeris"));

    let missing = verify_profile_assets(&manifest, "minimal", root);
    assert!(
        matches!(missing, Err(EphemerisError::KernelNotFound(message)) if message.contains("fixture") && message.contains("fixture.bsp"))
    );

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"fixture asset\n").unwrap();
    let verified = verify_profile_assets(&manifest, "minimal", root).unwrap();
    assert_eq!(verified.len(), 1);
    assert_eq!(verified[0].id, "fixture");
    assert_eq!(verified[0].status, AssetVerificationStatus::Valid);

    fs::write(&path, b"wrong\n").unwrap();
    let size_mismatch = verify_profile_assets(&manifest, "minimal", root);
    assert!(matches!(
        size_mismatch,
        Err(EphemerisError::AssetSizeMismatch { asset_id, .. }) if asset_id == "fixture"
    ));

    let optional = verify_profile_assets(&manifest, "optional", root).unwrap();
    assert_eq!(optional[0].status, AssetVerificationStatus::OptionalMissing);
}

#[test]
fn downloaded_object_listing_uses_only_valid_local_assets() {
    let manifest = fixture_asset_manifest();
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let fixture_path = root.join("kernels/fixture.bsp");
    let metadata_path = root.join("kernels/metadata.pca");
    fs::create_dir_all(fixture_path.parent().unwrap()).unwrap();
    fs::write(&fixture_path, b"fixture asset\n").unwrap();
    fs::write(&metadata_path, b"metadata\n").unwrap();

    let listed = downloaded_profile_objects(&manifest, "mixed", root).unwrap();

    assert_eq!(listed.objects.len(), 2);
    assert_eq!(listed.objects[0].object.id.as_str(), "earth");
    assert_eq!(listed.objects[0].object.name, "Earth");
    assert_eq!(listed.objects[0].source_asset_id, "fixture");
    assert_eq!(listed.objects[1].object.id.as_str(), "moon");
    assert_eq!(listed.skipped_assets.len(), 2);
    assert!(listed
        .skipped_assets
        .iter()
        .any(|asset| asset.id == "optional" && asset.reason == SkippedAssetReason::Missing));
    assert!(listed
        .skipped_assets
        .iter()
        .any(|asset| asset.id == "metadata" && asset.reason == SkippedAssetReason::NoCoverage));
}

#[test]
fn downloaded_object_listing_reports_invalid_assets_and_deduplicates_objects() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let first_path = root.join("kernels/first.bsp");
    let second_path = root.join("kernels/second.bsp");
    let bad_path = root.join("kernels/bad.bsp");
    fs::create_dir_all(first_path.parent().unwrap()).unwrap();
    fs::write(&first_path, b"fixture asset\n").unwrap();
    fs::write(&second_path, b"fixture asset\n").unwrap();
    fs::write(&bad_path, b"wrong\n").unwrap();

    let manifest = EphemerisAssetManifest::from_toml_str(
        r#"
version = 1

[profiles.dedup]
description = "Dedup profile"
assets = ["first", "second", "bad"]

[assets.first]
kind = "spk"
filename = "first.bsp"
source = "fixture"
url = "file:///tmp/first.bsp"
local_path = "kernels/first.bsp"
sha256 = "f0dad327e22e8cddc2e8057cf16d9b16ea6e36e87d31f46ee4d5943c69609c4f"
size_bytes = 14
required = true

[[assets.first.covers]]
id = "earth"
name = "Earth"
kind = "planet"
naif_id = 399

[assets.second]
kind = "spk"
filename = "second.bsp"
source = "fixture"
url = "file:///tmp/second.bsp"
local_path = "kernels/second.bsp"
sha256 = "f0dad327e22e8cddc2e8057cf16d9b16ea6e36e87d31f46ee4d5943c69609c4f"
size_bytes = 14
required = true

[[assets.second.covers]]
id = "earth"
name = "Earth"
kind = "planet"
naif_id = 399

[assets.bad]
kind = "spk"
filename = "bad.bsp"
source = "fixture"
url = "file:///tmp/bad.bsp"
local_path = "kernels/bad.bsp"
sha256 = "f0dad327e22e8cddc2e8057cf16d9b16ea6e36e87d31f46ee4d5943c69609c4f"
size_bytes = 14
required = true

[[assets.bad.covers]]
id = "mars"
name = "Mars"
kind = "planet"
naif_id = 499
"#,
    )
    .unwrap();

    let listed = downloaded_profile_objects(&manifest, "dedup", root).unwrap();

    assert_eq!(listed.objects.len(), 1);
    assert_eq!(listed.objects[0].object.id.as_str(), "earth");
    assert_eq!(listed.objects[0].source_asset_id, "first");
    assert_eq!(listed.skipped_assets.len(), 1);
    assert_eq!(listed.skipped_assets[0].id, "bad");
    assert!(matches!(
        &listed.skipped_assets[0].reason,
        SkippedAssetReason::Invalid(reason) if reason.contains("size mismatch")
    ));
}

#[test]
fn fetch_assets_can_use_local_fixture_source_without_internet() {
    let source_dir = tempfile::tempdir().unwrap();
    let asset_root = tempfile::tempdir().unwrap();
    let source_path = source_dir.path().join("fixture.bsp");
    fs::write(&source_path, b"fixture asset\n").unwrap();

    let manifest = EphemerisAssetManifest::from_toml_str(&format!(
        r#"
version = 1

[profiles.minimal]
description = "Fixture profile"
assets = ["fixture"]

[assets.fixture]
kind = "spk"
filename = "fixture.bsp"
source = "fixture"
url = "file://{}"
local_path = "kernels/fixture.bsp"
sha256 = "f0dad327e22e8cddc2e8057cf16d9b16ea6e36e87d31f46ee4d5943c69609c4f"
size_bytes = 14
required = true
"#,
        source_path.display()
    ))
    .unwrap();

    let fetched = fetch_profile_assets(&manifest, "minimal", asset_root.path(), false).unwrap();
    assert_eq!(fetched.len(), 1);
    assert_eq!(fetched[0].status, AssetVerificationStatus::Valid);
    assert_eq!(
        fs::read(asset_root.path().join("kernels/fixture.bsp")).unwrap(),
        b"fixture asset\n"
    );
}
