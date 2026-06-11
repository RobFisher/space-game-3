use space_game_ephemeris::{
    EphemerisError, EphemerisQuality, FrameId, GameTime, KernelManifest, ObjectKind,
    ObjectRegistry, SolarSystemBuilder, StateVector, Vec3Km, Vec3KmPerSec,
};

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

#[test]
fn kernel_manifest_parses_and_validates_without_downloads() {
    let manifest = KernelManifest::from_toml_str(
        r#"
schema_version = 1
profile = "standard"

[[kernels]]
id = "leapseconds"
kind = "lsk"
filename = "naif0012.tls"
url = "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/lsk/naif0012.tls"
sha256 = "abc123"
required = true
covers = "leap seconds"
"#,
    )
    .unwrap();

    assert_eq!(manifest.kernels.len(), 1);
    assert!(manifest.kernels[0].required);

    let invalid = KernelManifest::from_toml_str(
        r#"
schema_version = 99
profile = "standard"
"#,
    );
    assert!(matches!(
        invalid,
        Err(EphemerisError::InvalidObjectDefinition(_))
    ));
}
