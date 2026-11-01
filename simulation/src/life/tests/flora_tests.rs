//! Flore : compétition de Tilman, NPP limitée, feu stochastique, biomes et albédo.

use crate::life::flora::biome::{surface_albedo, whittaker_class};
use crate::life::flora::fire::{apply_fire, fire_probability};
use crate::life::flora::{step_cell, FloraCell, FloraEnv, FloraFlux, PFT_COUNT, PLANKTON};

fn env(temp_c: f64, precip_mm: f64) -> FloraEnv {
    FloraEnv {
        temp_c,
        precip_mm,
        soil_moisture: 0.5,
        npp_cap: 5000.0,
        n_avail: 20.0,
        p_avail: 3.0,
        co2_ppm: 280.0,
        is_ocean: false,
        neighbor_cover: [0.0; PFT_COUNT],
        fire_u: 1.0,
    }
}

fn settle(temp_c: f64, precip_mm: f64, years: usize) -> FloraCell {
    let mut cell = FloraCell::default();
    let mut e = env(temp_c, precip_mm);
    for _ in 0..years {
        e.neighbor_cover = cell.cover;
        step_cell(&mut cell, &e, 10.0);
    }
    cell
}

#[test]
fn cover_sum_stays_within_unity() {
    for (t, p) in [(26.0, 2500.0), (3.0, 500.0), (15.0, 800.0)] {
        let cell = settle(t, p, 300);
        let total: f64 = cell.cover.iter().sum();
        assert!((0.0..=1.0 + 1e-9).contains(&total), "T={t} P={p} couverture {total}");
        assert!(cell.cover.iter().all(|c| (0.0..=1.0).contains(c)));
    }
}

#[test]
fn climate_selects_the_climax_tree() {
    let tropical = settle(26.0, 2500.0, 300);
    assert!(tropical.cover[0] > 0.5, "arbre tropical : {:?}", tropical.cover);
    let boreal = settle(3.0, 500.0, 300);
    assert!(boreal.cover[2] > 0.5, "conifère : {:?}", boreal.cover);
    assert!(
        boreal.cover[0] < 1e-3,
        "arbre tropical hors climat : {:?}",
        boreal.cover
    );
}

#[test]
fn herbs_only_use_space_left_by_competitors() {
    let cell = settle(12.0, 900.0, 300);
    let above: f64 = cell.cover[..4].iter().sum();
    assert!(
        cell.cover[4] <= 1.0 - above + 1e-9,
        "herbe au-delà de l'espace libre : {:?}",
        cell.cover
    );
}

#[test]
fn no_nutrient_no_production() {
    let mut cell = FloraCell::default();
    let mut e = env(20.0, 1500.0);
    e.n_avail = 0.0;
    e.neighbor_cover = [0.5; PFT_COUNT];
    cell.cover = [0.5; PFT_COUNT];
    let flux: FloraFlux = step_cell(&mut cell, &e, 1.0);
    assert_eq!(flux.fixed_gc_m2, 0.0);
    assert!(cell.biomass_gc_m2.iter().all(|&b| b == 0.0));
}

#[test]
fn fire_is_stochastic_and_seeded_by_draw() {
    let p = fire_probability(500.0, 0.2, 1.0);
    assert!(p > 0.0 && p < 1.0, "probabilité de feu {p}");
    assert_eq!(fire_probability(0.0, 0.2, 1.0), 0.0);
    let mut cell = settle(22.0, 1200.0, 100);
    let before: f64 = cell.biomass_gc_m2.iter().sum();
    let mut flux = FloraFlux::default();
    assert!(!apply_fire(&mut cell, &env(22.0, 1200.0), 1.0, p, &mut flux));
    assert!(apply_fire(&mut cell, &env(22.0, 1200.0), 1.0, 0.0, &mut flux));
    let after: f64 = cell.biomass_gc_m2.iter().sum();
    assert!(after < before, "le feu doit consumer de la biomasse");
    assert!(flux.combusted_gc_m2 > 0.0 && flux.litter_in_gc_m2 > 0.0);
}

#[test]
fn ocean_cells_carry_only_plankton() {
    let mut cell = FloraCell::default();
    let mut e = env(15.0, 0.0);
    e.is_ocean = true;
    e.neighbor_cover = [0.0; PFT_COUNT];
    step_cell(&mut cell, &e, 10.0);
    assert!(cell.cover[PLANKTON] > 0.0);
    assert!(cell.cover[..PLANKTON].iter().all(|&c| c == 0.0));
}

#[test]
fn whittaker_known_points() {
    assert_eq!(whittaker_class(27.0, 2500.0, 0.8), 9, "forêt tropicale humide");
    assert_eq!(whittaker_class(-15.0, 100.0, 0.0), 0, "inlandsis");
    assert_eq!(whittaker_class(-3.0, 400.0, 0.3), 1, "toundra");
    assert_eq!(whittaker_class(3.0, 500.0, 0.5), 2, "taïga");
    assert_eq!(whittaker_class(8.0, 200.0, 0.2), 3, "steppe froide");
    assert_eq!(whittaker_class(15.0, 800.0, 0.7), 5, "forêt tempérée");
    assert_eq!(whittaker_class(25.0, 1000.0, 0.5), 7, "savane");
    assert_eq!(whittaker_class(25.0, 100.0, 0.0), 6, "désert chaud sans végétation");
}

#[test]
fn forest_is_darker_than_bare_ground() {
    let bare = surface_albedo(0.0, 0.0, false);
    let forest = surface_albedo(0.9, 0.0, false);
    let ice = surface_albedo(0.0, 1.0, false);
    assert!(forest < bare && bare < ice, "{forest} {bare} {ice}");
    assert!((surface_albedo(0.0, 0.0, true) - 0.06).abs() < 0.02, "océan ouvert");
}
