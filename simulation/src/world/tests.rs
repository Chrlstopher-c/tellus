use super::*;
use crate::clock::Speed;
use crate::matter::MatterConfig;
use crate::shared::units::{G_STANDARD, R_EARTH_M};
use std::time::Instant;

const SMALL_LEVEL: u32 = 3;
const SMALL_CELLS: usize = 642;
const L6_CELLS: usize = 40_962;

fn generated_json(seed: u64, level: u32) -> String {
    format!(
        r#"{{"source":{{"kind":"generated","seed":{seed},"waterPercent":70.8,"sizePercent":100,"distancePercent":100}},"level":{level}}}"#
    )
}

fn generated_world(seed: u64, level: u32) -> World {
    let request = WorldRequest::from_json(&generated_json(seed, level)).expect("requête valide");
    World::create(&request).expect("monde créé")
}

fn body_world(name: &str, level: u32) -> World {
    let json = format!(r#"{{"source":{{"kind":"body","name":"{name}"}},"level":{level}}}"#);
    World::create(&WorldRequest::from_json(&json).expect("requête valide")).expect("corps créé")
}

#[test]
fn name_matches_interface_algorithm() {
    let expected = [
        (0_u64, "Ordelron"),
        (1, "Oror"),
        (42, "Selmimi"),
        (12_345, "Ronka"),
        (4_294_967_295, "Arselthal"),
        (987_654_321, "Undel"),
    ];
    for (seed, name) in expected {
        assert_eq!(name_from_seed(seed), name, "graine {seed}");
    }
}

#[test]
fn request_defaults_level_and_reads_body() {
    let request = WorldRequest::from_json(r#"{"source":{"kind":"body","name":"Terre"}}"#).expect("valide");
    assert_eq!(request.level, DEFAULT_LEVEL);
    assert_eq!(request.source, Source::Body("Terre".to_owned()));
}

#[test]
fn request_rejects_bad_seed_and_unknown_kind() {
    let fractional =
        r#"{"source":{"kind":"generated","seed":1.5,"waterPercent":70,"sizePercent":100,"distancePercent":100}}"#;
    assert!(matches!(
        WorldRequest::from_json(fractional),
        Err(WorldError::InvalidRequest(_))
    ));
    let unknown = r#"{"source":{"kind":"galaxy"}}"#;
    assert!(matches!(
        WorldRequest::from_json(unknown),
        Err(WorldError::InvalidRequest(_))
    ));
    assert!(matches!(
        WorldRequest::from_json("{not json"),
        Err(WorldError::InvalidRequest(_))
    ));
}

#[test]
fn create_rejects_level_above_max() {
    let request = WorldRequest::from_json(&generated_json(1, MAX_LEVEL + 1)).expect("JSON valide");
    assert!(matches!(World::create(&request), Err(WorldError::InvalidLevel(_))));
}

#[test]
fn generated_world_reports_identity_and_cell_count() {
    let world = generated_world(42, SMALL_LEVEL);
    assert_eq!(world.info().cells, SMALL_CELLS);
    assert_eq!(world.info().seed, 42);
    assert_eq!(world.info().name, "Selmimi");
    assert_eq!(world.snapshot().len(), SMALL_CELLS * 4);
}

#[test]
fn generated_geometry_follows_config() {
    let spec = GeneratedSpec {
        seed: 1,
        water_percent: 70.8,
        size_percent: 50.0,
        distance_percent: 100.0,
    };
    let physics = physics::GeneratedPhysics::from_spec(&spec);
    assert!((physics.radius_m - R_EARTH_M * 0.5).abs() < 1e-3);
    assert!((physics.gravity_ms2 - G_STANDARD * 0.5).abs() < 1e-9);
    assert!((physics.solar_flux_wm2 - 1_361.0).abs() < 1e-9);
    let far = physics::GeneratedPhysics::from_spec(&GeneratedSpec {
        distance_percent: 200.0,
        ..spec
    });
    assert!((far.solar_flux_wm2 - 1_361.0 / 4.0).abs() < 1e-9);
}

#[test]
fn step_zero_changes_nothing_and_negative_is_rejected() {
    let mut world = generated_world(7, SMALL_LEVEL);
    assert_eq!(world.step(0.0), Ok(0.0));
    assert_eq!(world.ticks, 0);
    assert!(matches!(world.step(-1.0), Err(WorldError::InvalidDuration(_))));
    assert!(matches!(world.step(f64::NAN), Err(WorldError::InvalidDuration(_))));
}

#[test]
fn step_accumulates_time_and_keeps_fields_finite() {
    let mut world = generated_world(7, SMALL_LEVEL);
    assert_eq!(world.step(10.0), Ok(10.0));
    assert_eq!(world.step(100.0), Ok(110.0));
    assert_eq!(world.ticks, 2);
    assert!(world.snapshot().iter().all(|v| v.is_finite()));
}

#[test]
fn co2_comes_from_matter_and_o2_starts_at_zero() {
    let world = generated_world(7, SMALL_LEVEL);
    let vitals = world.vitals();
    assert!((vitals.co2_ppm - MatterConfig::default().co2_ppm).abs() < 1e-6);
    assert!(vitals.o2_percent.abs() < 1e-9);
    assert!(vitals.era.starts_with("Âge du monde · planète "));
}

#[test]
fn snapshot_carries_altitude_and_zero_life_fields() {
    let world = generated_world(7, SMALL_LEVEL);
    let fields = world.snapshot();
    for cell in 0..SMALL_CELLS {
        assert_eq!(fields[cell * 4], world.planet.elevation_m[cell]);
        assert_eq!(fields[cell * 4 + 3], 0.0);
    }
}

#[test]
fn inspect_rejects_out_of_range_cell() {
    let world = generated_world(7, SMALL_LEVEL);
    assert_eq!(
        world.inspect(SMALL_CELLS as u32).map(|_| ()),
        Err(WorldError::CellOutOfRange {
            cell: SMALL_CELLS as u32,
            cells: SMALL_CELLS
        })
    );
    let detail = world.inspect(0).expect("cellule 0");
    assert!(detail.lat_deg.abs() <= 90.0 && detail.lon_deg.abs() <= 180.0);
    assert!((0.0..=1.0).contains(&detail.soil.water));
    assert!((0.0..=1.0).contains(&detail.soil.carbon));
}

#[test]
fn creation_records_formation_event_once() {
    let mut world = generated_world(7, SMALL_LEVEL);
    let events = world.drain_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].title, "Formation du monde");
    assert_eq!(events[0].kind, "geology");
    assert_eq!(events[0].cell, None);
    assert!(world.drain_events().is_empty());
}

#[test]
fn lookup_has_texel_grid_size_and_valid_ids() {
    let world = generated_world(7, SMALL_LEVEL);
    let texels = (LOOKUP_WIDTH * LOOKUP_HEIGHT) as usize;
    assert_eq!(world.lookup().ids.len(), texels * 4);
    assert_eq!(world.lookup().weights.len(), texels * 2);
    let max_cell = world
        .lookup()
        .ids
        .chunks(4)
        .map(|c| c[0].max(c[1]).max(c[2]))
        .fold(0.0_f32, f32::max);
    assert!((max_cell as usize) < SMALL_CELLS);
}

#[test]
fn schedule_maps_rates_to_speeds() {
    assert_eq!(schedule::speed_for_rate(0.0), Ok(Speed::Pause));
    assert_eq!(schedule::speed_for_rate(100.0), Ok(Speed::Century));
    assert_eq!(schedule::speed_for_rate(1.0 / 365.25), Ok(Speed::Day));
    assert_eq!(schedule::speed_for_rate(1e7), Ok(Speed::TenMyr));
    assert!(matches!(
        schedule::speed_for_rate(42.0),
        Err(WorldError::InvalidRate(_))
    ));
    assert!(matches!(
        schedule::speed_for_rate(f64::NAN),
        Err(WorldError::InvalidRate(_))
    ));
}

#[test]
fn schedule_produces_century_ticks_from_wall_time() {
    let mut world = generated_world(7, SMALL_LEVEL);
    assert_eq!(world.schedule(0.0, 0.0, 0.0), Ok(0.0));
    let dt = world.schedule(0.2, 100.0, 0.0).expect("vitesse valide");
    assert!((dt - 10.0).abs() < 1e-9, "dt = {dt}");
}

#[test]
fn real_body_without_ocean_is_all_land() {
    let world = body_world("Mars", 6);
    assert_eq!(world.info().cells, L6_CELLS);
    assert!(world.is_ocean.iter().all(|&ocean| !ocean));
    assert_eq!(world.info().name, "Mars");
}

#[test]
fn real_body_rejects_unknown_name_and_unsupported_level() {
    let unknown = WorldRequest::from_json(r#"{"source":{"kind":"body","name":"Pluton"},"level":6}"#).expect("JSON");
    assert!(matches!(World::create(&unknown), Err(WorldError::UnknownBody(_))));
    let level_3 = WorldRequest::from_json(r#"{"source":{"kind":"body","name":"Terre"},"level":3}"#).expect("JSON");
    assert!(World::create(&level_3).is_err());
}

#[test]
#[ignore = "mesure de performance : cargo test --release tick_cost_l6 -- --ignored --nocapture"]
fn tick_cost_l6_at_century_speed() {
    let mut world = generated_world(42, 6);
    world.step(10.0).expect("premier pas");
    let rounds = 20_u32;
    let start = Instant::now();
    for _ in 0..rounds {
        world.step(10.0).expect("pas de 10 ans");
    }
    let per_tick = start.elapsed().as_secs_f64() / f64::from(rounds);
    let vitals = world.vitals();
    println!("tick L6 à 100 ans/s (Δt = 10 ans) : {:.1} ms", per_tick * 1e3);
    println!(
        "après {:.0} ans : T̄ {:.2} °C, CO₂ {:.0} ppm, O₂ {:.3} %",
        world.time_years(),
        vitals.mean_temp_c,
        vitals.co2_ppm,
        vitals.o2_percent
    );
}

#[test]
fn playable_bodies_create_and_step() {
    for name in ["Mercure", "Vénus", "Terre", "Mars"] {
        let mut world = body_world(name, 6);
        world.step(10.0).unwrap_or_else(|e| panic!("{name} : {e}"));
        let vitals = world.vitals();
        assert!(vitals.mean_temp_c.is_finite(), "{name}");
        assert!(vitals.co2_ppm.is_finite() && vitals.co2_ppm >= 0.0, "{name}");
    }
}

#[test]
fn moon_refused_explicitly_until_climate_has_a_target() {
    let json = r#"{"source":{"kind":"body","name":"Lune"},"level":6}"#;
    let request = WorldRequest::from_json(json).expect("JSON valide");
    assert!(matches!(World::create(&request), Err(WorldError::Greenhouse(_))));
}
