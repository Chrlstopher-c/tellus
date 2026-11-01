//! Contrat de la genèse : tests de `docs/recherche/genese.md` (section « Tests de validation », G1 à G8).
//! Les tests tournent à L6 (40 962 cellules) ; le budget L7 est un test `#[ignore]` en release.

use std::collections::HashSet;
use std::sync::OnceLock;
use std::time::Instant;

use super::g1_plates::PlateMap;
use super::g2_motion::surface_velocity;
use super::g3_boundaries::{classify_motion, BoundaryKind};
use super::g4_profiles::profile_for;
use super::g5_depth::{airy_elevation_m, gdh1_depth_m};
use super::g6_erosion::erode;
use super::g7_sea_level::sea_level_m;
use super::vec3::{dot, norm};
use super::*;
use crate::shared::grid::Grid;
use crate::shared::units::R_EARTH_M;

const TEST_LEVEL: u8 = 6;
const TEST_SEED: u64 = 42;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn params(seed: u64, level: u8) -> WorldParams {
    WorldParams {
        seed,
        level,
        radius_m: R_EARTH_M,
        ..WorldParams::default()
    }
}

struct Reference {
    grid: Grid,
    planet: Planet,
}

fn reference() -> &'static Reference {
    static REFERENCE: OnceLock<Reference> = OnceLock::new();
    REFERENCE.get_or_init(|| {
        let grid = Grid::new(TEST_LEVEL, R_EARTH_M);
        let planet = generate(&grid, &params(TEST_SEED, TEST_LEVEL)).expect("genèse de référence");
        Reference { grid, planet }
    })
}

fn fnv_feed(mut hash: u64, bytes: &[u8]) -> u64 {
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn fnv_f32(hash: u64, values: &[f32]) -> u64 {
    values.iter().fold(hash, |h, v| fnv_feed(h, &v.to_bits().to_le_bytes()))
}

fn planet_hash(planet: &Planet) -> u64 {
    let mut hash = FNV_OFFSET;
    for field in [&planet.elevation_m, &planet.crust, &planet.plate, &planet.age_ma] {
        hash = fnv_f32(hash, field);
    }
    for plate in &planet.plates {
        for axis in plate.pole {
            hash = fnv_feed(hash, &axis.to_bits().to_le_bytes());
        }
        hash = fnv_feed(hash, &plate.omega_rad_s.to_bits().to_le_bytes());
    }
    hash
}

/// Fraction d'aire (pondérée par les aires exactes de la grille) des cellules vérifiant `keep`.
fn area_fraction(grid: &Grid, keep: impl Fn(usize) -> bool) -> f64 {
    let total: f64 = grid.areas_m2().iter().sum();
    let kept: f64 = (0..grid.cell_count())
        .filter(|&c| keep(c))
        .map(|c| grid.areas_m2()[c])
        .sum();
    kept / total
}

fn area_mean(grid: &Grid, cells: impl Iterator<Item = (usize, f64)>) -> f64 {
    let (mut weighted, mut total) = (0.0, 0.0);
    for (c, value) in cells {
        weighted += grid.areas_m2()[c] * value;
        total += grid.areas_m2()[c];
    }
    weighted / total
}

// ---------------------------------------------------------------- déterminisme (transverse)

#[test]
fn genesis_deterministic() {
    let grid = Grid::new(TEST_LEVEL, R_EARTH_M);
    let first = generate(&grid, &params(TEST_SEED, TEST_LEVEL)).expect("première exécution");
    let second = generate(&grid, &params(TEST_SEED, TEST_LEVEL)).expect("seconde exécution");
    assert_eq!(planet_hash(&first), planet_hash(&second));
}

#[test]
fn genesis_seed_diversity() {
    let grid = Grid::new(TEST_LEVEL, R_EARTH_M);
    let hashes: HashSet<u64> = (1..=10_u64)
        .map(|seed| planet_hash(&generate(&grid, &params(seed, TEST_LEVEL)).expect("graine")))
        .collect();
    assert_eq!(hashes.len(), 10);
}

#[test]
fn genesis_finite() {
    let planet = &reference().planet;
    for field in [&planet.elevation_m, &planet.crust, &planet.plate, &planet.age_ma] {
        assert!(field.iter().all(|v| v.is_finite()));
    }
}

#[test]
fn invalid_params_rejected() {
    let grid = Grid::new(TEST_LEVEL, R_EARTH_M);
    let wrong_level = generate(&grid, &params(TEST_SEED, 7));
    assert!(matches!(wrong_level, Err(GenesisError::LevelMismatch { .. })));
    let bad_water = WorldParams {
        water_fraction: 1.5,
        ..params(TEST_SEED, TEST_LEVEL)
    };
    assert!(matches!(
        generate(&grid, &bad_water),
        Err(GenesisError::InvalidParam(_))
    ));
}

// ---------------------------------------------------------------- plaques et mouvement (G1, G2)

#[test]
fn plate_count() {
    let count = reference().planet.plates.len();
    assert!((8..=14).contains(&count), "{count} plaques");
}

#[test]
fn plate_connected() {
    let Reference { grid, planet } = reference();
    for (id, _) in planet.plates.iter().enumerate() {
        let members: Vec<usize> = (0..grid.cell_count())
            .filter(|&c| planet.plate[c] == id as f32)
            .collect();
        assert!(!members.is_empty(), "plaque {id} vide");
        let mut seen = vec![false; grid.cell_count()];
        let mut stack = vec![members[0]];
        seen[members[0]] = true;
        let mut reached = 1_usize;
        while let Some(cell) = stack.pop() {
            for &nb in grid.neighbors(cell) {
                let nb = nb as usize;
                if !seen[nb] && planet.plate[nb] == id as f32 {
                    seen[nb] = true;
                    reached += 1;
                    stack.push(nb);
                }
            }
        }
        assert_eq!(reached, members.len(), "plaque {id} non connexe");
    }
}

#[test]
fn plate_types() {
    let crust = &reference().planet.crust;
    assert!(crust.contains(&0.0), "aucune croûte océanique");
    assert!(crust.contains(&1.0), "aucune croûte continentale");
}

#[test]
fn velocity_tangent() {
    let Reference { grid, planet } = reference();
    for plate in &planet.plates {
        for &x in grid.centers() {
            let v = surface_velocity(plate, x, R_EARTH_M);
            assert!(dot(v, x).abs() <= 1e-9 * norm(v) + 1e-12);
        }
    }
}

#[test]
fn velocity_range() {
    for plate in &reference().planet.plates {
        let speed = plate.speed_cm_per_year(R_EARTH_M);
        assert!((1.0 - 1e-9..=15.0 + 1e-9).contains(&speed), "{speed} cm/an");
    }
}

// ---------------------------------------------------------------- frontières et profils (G3, G4)

#[test]
fn boundary_convergent() {
    let n = [1.0, 0.0, 0.0];
    let (kind, rate) = classify_motion([0.02, 0.0, 0.0], n);
    assert_eq!(kind, BoundaryKind::Convergent);
    assert!(rate > 0.0);
}

#[test]
fn boundary_divergent() {
    let n = [1.0, 0.0, 0.0];
    let (kind, rate) = classify_motion([-0.02, 0.0, 0.0], n);
    assert_eq!(kind, BoundaryKind::Divergent);
    assert!(rate < 0.0);
}

#[test]
fn boundary_transform() {
    let n = [1.0, 0.0, 0.0];
    let (kind, _) = classify_motion([0.0, 0.02, 0.0], n);
    assert_eq!(kind, BoundaryKind::Transform);
}

#[test]
fn trench_depth() {
    let trench = profile_for(BoundaryKind::Convergent, false, true, true);
    assert!(
        (-8000.0..=-6000.0).contains(&trench.amplitude_m),
        "{}",
        trench.amplitude_m
    );
}

#[test]
fn orogen_height() {
    let collision = profile_for(BoundaryKind::Convergent, true, true, false);
    assert!(
        (3000.0..=5000.0).contains(&collision.amplitude_m),
        "{}",
        collision.amplitude_m
    );
}

// ---------------------------------------------------------------- âge, profondeur, isostasie (G5)

#[test]
fn gdh1_continuity() {
    let below = gdh1_depth_m(20.0 - 1e-9);
    let above = gdh1_depth_m(20.0 + 1e-9);
    assert!((below - above).abs() < 5.0, "{below} / {above}");
}

#[test]
fn gdh1_values() {
    assert!((gdh1_depth_m(0.0) - 2600.0).abs() < 1e-9);
    let at_100 = gdh1_depth_m(100.0);
    assert!((5450.0..=5550.0).contains(&at_100), "{at_100}");
}

#[test]
fn airy_isostasy() {
    let elevation = airy_elevation_m(45_000.0);
    assert!((elevation - 1818.0).abs() <= 1.0, "{elevation}");
}

#[test]
fn oceanic_depth_mean() {
    let Reference { grid, planet } = reference();
    let mean = area_mean(
        grid,
        (0..grid.cell_count())
            .filter(|&c| planet.elevation_m[c] < 0.0)
            .map(|c| (c, -f64::from(planet.elevation_m[c]))),
    );
    assert!(
        (3400.0..=4000.0).contains(&mean),
        "profondeur océanique moyenne {mean} m"
    );
}

// ---------------------------------------------------------------- érosion (G6)

#[test]
fn erosion_mass_balance() {
    let grid = Grid::new(TEST_LEVEL, R_EARTH_M);
    let stage = assemble(&grid, &params(TEST_SEED, TEST_LEVEL)).expect("étapes G1 à G5");
    let mut heights = stage.heights_m.clone();
    let base = sea_level_m(&heights, grid.areas_m2(), 0.708);
    let report = erode(&grid, &stage.continental, &mut heights, base);
    assert!(report.removed_m3 > 0.0, "érosion nulle");
    let gap = (report.removed_m3 - report.deposited_m3).abs() / report.removed_m3;
    assert!(gap < 1e-6, "écart relatif {gap}");
}

#[test]
fn erosion_monotone() {
    let grid = Grid::new(TEST_LEVEL, R_EARTH_M);
    let stage = assemble(&grid, &params(TEST_SEED, TEST_LEVEL)).expect("étapes G1 à G5");
    let mut eroded = stage.heights_m.clone();
    let base = sea_level_m(&eroded, grid.areas_m2(), 0.708);
    erode(&grid, &stage.continental, &mut eroded, base);
    for (cell, (&before, &after)) in stage.heights_m.iter().zip(&eroded).enumerate() {
        if stage.continental[cell] {
            assert!(after <= before + 1e-9, "cellule {cell} remontée : {before} -> {after}");
        }
    }
}

// ---------------------------------------------------------------- niveau de la mer et hypsométrie (G7)

#[test]
fn sea_fraction() {
    let Reference { grid, planet } = reference();
    let sea = area_fraction(grid, |c| planet.elevation_m[c] < 0.0);
    assert!((sea - 0.708).abs() <= 0.005, "fraction d'eau {sea}");
}

#[test]
fn land_fraction() {
    let Reference { grid, planet } = reference();
    let land = area_fraction(grid, |c| planet.elevation_m[c] >= 0.0);
    assert!((0.262..=0.322).contains(&land), "fraction de terres {land}");
}

#[test]
fn land_mean_elevation() {
    let Reference { grid, planet } = reference();
    let mean = area_mean(
        grid,
        (0..grid.cell_count())
            .filter(|&c| planet.elevation_m[c] >= 0.0)
            .map(|c| (c, f64::from(planet.elevation_m[c]))),
    );
    assert!((690.0..=990.0).contains(&mean), "altitude moyenne des terres {mean} m");
}

#[test]
fn ocean_mean_depth() {
    let Reference { grid, planet } = reference();
    let mean = area_mean(
        grid,
        (0..grid.cell_count())
            .filter(|&c| planet.elevation_m[c] < 0.0)
            .map(|c| (c, -f64::from(planet.elevation_m[c]))),
    );
    assert!(
        (3400.0..=4000.0).contains(&mean),
        "profondeur moyenne des océans {mean} m"
    );
}

const BIN_M: f64 = 100.0;
const HYPSO_MIN_M: f64 = -6000.0;
const HYPSO_BINS: usize = 120;

/// Histogramme pondéré par aire (pas de 100 m), lissé sur 3 classes.
fn hypsometry(grid: &Grid, planet: &Planet) -> Vec<f64> {
    let mut bins = vec![0.0_f64; HYPSO_BINS];
    let total: f64 = grid.areas_m2().iter().sum();
    for cell in 0..grid.cell_count() {
        let index = ((f64::from(planet.elevation_m[cell]) - HYPSO_MIN_M) / BIN_M).floor();
        let index = index.clamp(0.0, (HYPSO_BINS - 1) as f64) as usize;
        bins[index] += grid.areas_m2()[cell] / total;
    }
    (0..HYPSO_BINS)
        .map(|i| {
            let lo = i.saturating_sub(1);
            let hi = (i + 1).min(HYPSO_BINS - 1);
            bins[lo..=hi].iter().sum::<f64>() / (hi - lo + 1) as f64
        })
        .collect()
}

fn bin_center_m(index: usize) -> f64 {
    HYPSO_MIN_M + (index as f64 + 0.5) * BIN_M
}

fn argmax_in(density: &[f64], low_m: f64, high_m: f64) -> usize {
    (0..HYPSO_BINS)
        .filter(|&i| (low_m..=high_m).contains(&bin_center_m(i)))
        .max_by(|&a, &b| density[a].total_cmp(&density[b]))
        .expect("classe dans la fenêtre")
}

#[test]
fn hypsometry_bimodal() {
    let Reference { grid, planet } = reference();
    let density = hypsometry(grid, planet);
    let land_peak = argmax_in(&density, -200.0, 400.0);
    let sea_peak = argmax_in(&density, -5000.0, -3500.0);
    let (lo, hi) = (land_peak.min(sea_peak), land_peak.max(sea_peak));
    let trough = density[lo..=hi].iter().copied().fold(f64::INFINITY, f64::min);
    let lower_peak = density[land_peak].min(density[sea_peak]);
    assert!(trough < 0.5 * lower_peak, "creux {trough} vs pics {lower_peak}");
}

// ---------------------------------------------------------------- figement et budget

#[test]
fn tectonics_frozen_mvp() {
    let grid = Grid::new(TEST_LEVEL, R_EARTH_M);
    let first = generate(&grid, &params(TEST_SEED, TEST_LEVEL)).expect("première");
    let again = generate(&grid, &params(TEST_SEED, TEST_LEVEL)).expect("seconde");
    assert_eq!(first.plates, again.plates);
    assert_eq!(planet_hash(&first), planet_hash(&again));
}

#[test]
#[ignore]
fn genesis_budget() {
    let grid = Grid::new(7, R_EARTH_M);
    let start = Instant::now();
    let planet = generate(&grid, &params(TEST_SEED, 7)).expect("genèse L7");
    let elapsed = start.elapsed().as_secs_f64();
    eprintln!("genèse L7 : {elapsed:.3} s");
    assert_eq!(planet.elevation_m.len(), grid.cell_count());
    assert!(elapsed < 5.0, "genèse L7 trop lente : {elapsed:.3} s");
}

#[test]
fn plate_map_covers_all_cells() {
    let grid = Grid::new(TEST_LEVEL, R_EARTH_M);
    let map: PlateMap = super::g1_plates::partition(&grid, TEST_SEED).expect("partition");
    assert_eq!(map.plate_of.len(), grid.cell_count());
    assert!(map.plate_of.iter().all(|&p| (p as usize) < map.continental.len()));
}
