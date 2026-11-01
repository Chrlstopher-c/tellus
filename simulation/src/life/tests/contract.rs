//! Contrat du module : pas de vie préfaite, déterminisme, conservation, garde « aucun script », banc de performance.

use super::support::{carnivore_traits, grid, herbivore_traits, producer_traits, run, temperate_forcing, world};
use crate::life::{init, spawn_lineage};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn seed_community(state: &mut crate::life::LifeState, cells: usize) {
    spawn_lineage(state, producer_traits(), cells / 3).expect("cellule valide");
    spawn_lineage(state, herbivore_traits(), cells / 3).expect("cellule valide");
    spawn_lineage(state, carnivore_traits(), cells / 2).expect("cellule valide");
}

#[test]
fn no_life_no_flora_no_fauna() {
    let (grid, mut state, forcing) = world(4);
    let mut report = run(&mut state, &grid, &forcing, 0);
    for t in 1..200 {
        report = run(&mut state, &grid, &forcing, t);
    }
    assert!(!report.photosynth_present);
    assert!(state.biomass_gc_m2().iter().all(|&b| b == 0.0));
    assert!(state.veg_cover().iter().all(|&c| c == 0.0));
    assert_eq!(state.lineages().alive_count(), 0);
    assert_eq!(report.fauna_biomass_kgc, 0.0);
}

#[test]
fn rejects_inconsistent_inputs_and_grid() {
    let (grid, mut state, forcing) = world(4);
    let other = super::support::grid(3);
    let report = crate::life::step(
        &mut state,
        &other,
        &forcing.inputs(),
        1.0,
        crate::clock::Regime::Annual,
        0,
        1,
    );
    assert!(report.input_error);
    let bad_dt = crate::life::step(
        &mut state,
        &grid,
        &forcing.inputs(),
        0.0,
        crate::clock::Regime::Annual,
        0,
        1,
    );
    assert!(bad_dt.input_error);
}

#[test]
fn spawn_lineage_rejects_invalid_cell() {
    let (grid, mut state, _forcing) = world(4);
    assert!(spawn_lineage(&mut state, herbivore_traits(), grid.cell_count()).is_none());
    assert!(spawn_lineage(&mut state, herbivore_traits(), 0).is_some());
}

#[test]
fn photosynthetic_lineage_enables_flora() {
    let (grid, mut state, forcing) = world(4);
    let centre = grid.cell_count() / 2;
    spawn_lineage(&mut state, producer_traits(), centre).expect("cellule valide");
    let mut report = run(&mut state, &grid, &forcing, 0);
    for t in 1..120 {
        report = run(&mut state, &grid, &forcing, t);
    }
    assert!(report.photosynth_present);
    assert!(
        state.biomass_gc_m2().iter().any(|&b| b > 0.0),
        "aucune biomasse végétale"
    );
}

#[test]
fn deterministic_across_thread_counts() {
    let fingerprint = |threads: usize| {
        let (grid, mut state, forcing) = world(4);
        state.set_threads(threads);
        seed_community(&mut state, grid.cell_count());
        for t in 0..150 {
            run(&mut state, &grid, &forcing, t);
        }
        state.fingerprint()
    };
    assert_eq!(fingerprint(1), fingerprint(4));
    assert_eq!(fingerprint(1), fingerprint(7));
}

#[test]
fn carbon_nitrogen_phosphorus_conserved_per_tick() {
    let (grid, mut state, forcing) = world(4);
    state.set_threads(2);
    seed_community(&mut state, grid.cell_count());
    for t in 0..200 {
        let report = run(&mut state, &grid, &forcing, t);
        assert!(
            report.conservation_error <= 1e-9,
            "tick {t}: {}",
            report.conservation_error
        );
        let ledger = state.ledger();
        for pools in [&ledger.carbon, &ledger.nitrogen, &ledger.phosphorus] {
            assert!(
                pools.total().abs() <= 1e-9 * (1.0 + pools.magnitude()),
                "tick {t}: {pools:?}"
            );
        }
    }
}

#[test]
fn population_bounded_short_run() {
    let (grid, mut state, forcing) = world(4);
    state.set_threads(2);
    seed_community(&mut state, grid.cell_count());
    for t in 0..1000 {
        let report = run(&mut state, &grid, &forcing, t);
        assert!(report.fauna_biomass_kgc.is_finite());
    }
    for cell in 0..state.cell_count() {
        assert!(state.cohorts(cell).len() <= crate::life::fauna::MAX_COHORTS);
        assert!(state.cohorts(cell).iter().all(|c| c.n.is_finite() && c.n >= 0.0));
    }
}

#[test]
fn no_script_in_life() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/life");
    let mut files = Vec::new();
    collect_sources(&root, &mut files);
    assert!(files.len() >= 10, "sources du vivant introuvables");
    let forbidden = concat!("tick", " ==");
    for file in files {
        let text = fs::read_to_string(&file).expect("lecture d'une source du vivant");
        assert!(!text.contains(forbidden), "date ou script codé dans {}", file.display());
    }
}

fn collect_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n != "tests") {
                collect_sources(&path, out);
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
#[ignore = "long : 10 000 ans simulés, à lancer en release"]
fn population_bounded_10kyr() {
    let (grid, mut state, forcing) = world(4);
    state.set_threads(4);
    seed_community(&mut state, grid.cell_count());
    let mut peak = 0.0_f64;
    for t in 0..10_000 {
        let report = run(&mut state, &grid, &forcing, t);
        assert!(report.fauna_biomass_kgc.is_finite());
        peak = peak.max(report.fauna_biomass_kgc);
    }
    assert!(peak.is_finite());
}

/// Banc : un tick à L6 (< 80 ms) et L7 (< 300 ms) en release, 6 threads. Moyenne sur 3 ticks après échauffement.
#[test]
#[ignore = "banc : à lancer en release, sur la machine cible"]
fn bench_tick_l6_l7_release() {
    for (level, budget_ms) in [(6_u8, 80.0_f64), (7, 300.0)] {
        let grid = grid(level);
        let mut state = init(&grid);
        state.set_threads(6);
        let forcing = temperate_forcing(&grid);
        seed_community(&mut state, grid.cell_count());
        for t in 0..10 {
            run(&mut state, &grid, &forcing, t);
        }
        let start = Instant::now();
        for t in 10..13 {
            run(&mut state, &grid, &forcing, t);
        }
        let ms = start.elapsed().as_secs_f64() * 1000.0 / 3.0;
        eprintln!("bench L{level} : {ms:.1} ms par tick (budget {budget_ms} ms)");
        assert!(ms < budget_ms, "L{level} : {ms:.1} ms > {budget_ms} ms");
    }
}
