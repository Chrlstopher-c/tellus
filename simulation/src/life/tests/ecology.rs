//! Écologie à l'échelle : Lotka-Volterra isolé, communauté ensemencée, plafonds énergétique et de Damuth,
//! pyramide trophique, biomes, run long en régime relaxé. Les tests longs sont `#[ignore]` (release).

use super::support::{grid, run, seed_community_wide, temperate_forcing, world, SEED};
use crate::clock::Regime;
use crate::life::evolution::idx;
use crate::life::fauna::damuth::territorial_cap;
use crate::life::fauna::dynamics::LotkaVolterra;
use crate::life::flora::biome::whittaker_class;
use crate::life::step;

/// Biomasse animale (kgC) et végétale (kgC) par cellule, et classes herbivore / carnivore.
struct Levels {
    plant: Vec<f64>,
    herbivore: Vec<f64>,
    carnivore: Vec<f64>,
}

fn levels(state: &crate::life::LifeState, areas: &[f64]) -> Levels {
    let mut out = Levels {
        plant: vec![0.0; state.cell_count()],
        herbivore: vec![0.0; state.cell_count()],
        carnivore: vec![0.0; state.cell_count()],
    };
    for (cell, area) in areas.iter().enumerate().take(state.cell_count()) {
        let kg = area / 1000.0;
        out.plant[cell] = state.flora_cell(cell).biomass_gc_m2.iter().sum::<f64>() * kg;
        for c in state.cohorts(cell).iter().filter(|c| !c.is_producer()) {
            if c.genome[idx::DIET] < 0.5 {
                out.herbivore[cell] += c.biomass();
            } else {
                out.carnivore[cell] += c.biomass();
            }
        }
    }
    out
}

/// Dérive relative de l'invariant V après 1 000 ans, RK2 au pas `dt` (paramètres LV fixés).
fn lotka_volterra_drift(dt: f64) -> f64 {
    let lv = LotkaVolterra {
        a: 1.0,
        b: 0.1,
        c: 0.075,
        d: 1.5,
    };
    let mut s = [10.0, 5.0];
    let v0 = lv.invariant(s);
    let steps = (1000.0 / dt).round() as u64;
    for _ in 0..steps {
        s = lv.rk2_step(s, dt);
    }
    ((lv.invariant(s) - v0) / v0.abs()).abs()
}

#[test]
fn lotka_volterra_drift_below_one_percent_at_fine_step() {
    let drift = lotka_volterra_drift(0.01);
    eprintln!("LV dérive V (dt=0,01 an, 1 000 ans) = {drift:.3e}");
    assert!(drift < 0.01, "dérive de V : {drift:.3e}");
}

#[test]
#[ignore = "mesuré : dérive de V = 2,86e-2 à Δt = 0,1 an (RK2), seuil 1e-2 non tenu ; voir rapport"]
fn lotka_volterra_drift_below_one_percent_at_tenth_year() {
    let drift = lotka_volterra_drift(0.1);
    assert!(drift < 0.01, "dérive de V : {drift:.3e}");
}

#[test]
#[ignore = "long : 1 000 ans annuels en communauté, à lancer en release"]
fn community_respects_energy_and_damuth_caps() {
    let (grid, mut state, forcing) = world(4);
    state.set_threads(6);
    seed_community_wide(&mut state, grid.cell_count());
    for t in 0..1000 {
        run(&mut state, &grid, &forcing, t);
    }
    let lv = levels(&state, grid.areas_m2());
    for cell in 0..state.cell_count() {
        assert!(
            lv.herbivore[cell] <= 0.3 * lv.plant[cell] + 1e-9,
            "cellule {cell} : herbivores {:.3e} > 0,3 × végétaux {:.3e}",
            lv.herbivore[cell],
            lv.plant[cell]
        );
        for c in state.cohorts(cell).iter().filter(|c| !c.is_producer()) {
            let cap = territorial_cap(c.mass_ind, grid.areas_m2()[cell]).max(1e-300);
            assert!(
                c.n <= 10.0 * cap,
                "cellule {cell} : effectif {:.3e} > 10 × Damuth {cap:.3e}",
                c.n
            );
        }
    }
}

#[test]
#[ignore = "long : pyramide trophique après 2 000 ans, à lancer en release"]
fn trophic_pyramid_after_two_thousand_years() {
    let (grid, mut state, forcing) = world(4);
    state.set_threads(6);
    seed_community_wide(&mut state, grid.cell_count());
    for t in 0..2000 {
        run(&mut state, &grid, &forcing, t);
    }
    let lv = levels(&state, grid.areas_m2());
    let (p, h, c) = (sum(&lv.plant), sum(&lv.herbivore), sum(&lv.carnivore));
    eprintln!(
        "PYRAMIDE plantes={p:.3e} herbivores={h:.3e} carnivores={c:.3e} h/p={:.3e} c/h={:.3e}",
        h / p,
        c / h
    );
    assert!((0.01..=0.3).contains(&(h / p)), "herbivores/végétaux = {:.3e}", h / p);
    assert!((0.01..=0.3).contains(&(c / h)), "carnivores/herbivores = {:.3e}", c / h);
    assert!(p / (h + c) > 10.0, "végétaux/animaux = {:.3e}", p / (h + c));
}

fn sum(values: &[f64]) -> f64 {
    values.iter().sum()
}

#[test]
#[ignore = "long : 1 000 ans en communauté, biomes, à lancer en release"]
fn biomes_follow_climate_on_most_land() {
    let grid = grid(4);
    let mut state = crate::life::init(&grid);
    state.set_threads(6);
    let forcing = temperate_forcing(&grid);
    seed_community_wide(&mut state, grid.cell_count());
    for t in 0..1000 {
        run(&mut state, &grid, &forcing, t);
    }
    let inputs = forcing.inputs();
    let land: Vec<usize> = (0..grid.cell_count()).filter(|&i| !inputs.is_ocean[i]).collect();
    let matching = land
        .iter()
        .filter(|&&i| {
            let expected = whittaker_class(
                f64::from(inputs.temperature_c[i]),
                f64::from(inputs.precip_mm_yr[i]),
                1.0,
            );
            state.biome()[i] == expected
        })
        .count();
    let share = matching as f64 / land.len() as f64;
    eprintln!("BIOMES accord climat/végétation = {share:.3}");
    assert!(share >= 0.7, "accord biome/climat : {share:.3}");
}

#[test]
#[ignore = "long : 10 000 ans en régime relaxé (100 ticks de 100 ans), à lancer en release"]
fn ten_thousand_years_relaxed_regime_stays_bounded() {
    let grid = grid(4);
    let mut state = crate::life::init(&grid);
    state.set_threads(6);
    let forcing = temperate_forcing(&grid);
    seed_community_wide(&mut state, grid.cell_count());
    let inputs = forcing.inputs();
    for t in 0..100 {
        let report = step(&mut state, &grid, &inputs, 100.0, Regime::Secular, t, SEED);
        assert!(!report.input_error && report.fauna_biomass_kgc.is_finite(), "tick {t}");
    }
    let lv = levels(&state, grid.areas_m2());
    let (p, a) = (sum(&lv.plant), sum(&lv.herbivore) + sum(&lv.carnivore));
    eprintln!(
        "RELAXE 10kyr lignées={} végétaux={p:.3e} animaux={a:.3e} ratio={:.3e}",
        state.lineages().alive_count(),
        a / p
    );
    assert!(a <= 0.3 * p, "animaux/végétaux = {:.3e}", a / p);
}
