//! G1 : 8 à 14 plaques par croissance de Dijkstra multi-source ; frontières irrégulières grâce à un bruit
//! basse fréquence sur le coût d'entrée. Chaque plaque est connexe par construction (arbre de croissance).

use super::graph::{arc_m, grow, UNREACHED};
use super::noise::fbm;
use super::vec3::dot;
use super::GenesisError;
use crate::shared::grid::Grid;
use crate::shared::rng::{hash, unit_f64};

pub(super) const STEP_ID: u64 = 1;
const PLATE_MIN: usize = 8;
const PLATE_MAX: usize = 14;
const SEED_CANDIDATES: u32 = 8;
/// Surface couverte par les plaques continentales (croûte continentale terrestre ≈ 40 %).
const CONTINENTAL_AREA_TARGET: f64 = 0.40;
const NOISE_FREQ: f64 = 2.0;
const NOISE_OCTAVES: u32 = 4;
const NOISE_AMPLITUDE: f64 = 0.4;
const STREAM_COUNT: u32 = 0;
const STREAM_SEED: u32 = 1;
const STREAM_TYPE: u32 = 2;
const STREAM_NOISE: u32 = 3;

/// Partition de la grille : plaque de chaque cellule et type de croûte de chaque plaque.
pub(super) struct PlateMap {
    pub(super) plate_of: Vec<u32>,
    pub(super) continental: Vec<bool>,
}

pub(super) fn partition(grid: &Grid, seed: u64) -> Result<PlateMap, GenesisError> {
    let count = plate_count(seed);
    let seeds = pick_seeds(grid, seed, count);
    let noise_seed = hash(seed, STEP_ID, 0, STREAM_NOISE);
    let centers = grid.centers();
    let weights: Vec<f64> = centers
        .iter()
        .map(|&x| 1.0 + NOISE_AMPLITUDE * fbm(noise_seed, x, NOISE_FREQ, NOISE_OCTAVES))
        .collect();
    let field = grow(grid, &seeds, |from, to| Some(arc_m(grid, from, to) * weights[to]));
    if field.origin.contains(&UNREACHED) {
        return Err(GenesisError::GridDisconnected);
    }
    let continental = assign_types(grid, seed, &field.origin, count);
    Ok(PlateMap {
        plate_of: field.origin,
        continental,
    })
}

/// Nombre de plaques, tiré uniformément dans [8, 14].
pub(super) fn plate_count(seed: u64) -> usize {
    let u = unit_f64(seed, STEP_ID, 0, STREAM_COUNT);
    PLATE_MIN + (u * (PLATE_MAX - PLATE_MIN + 1) as f64) as usize
}

/// Germes espacés : pour chaque plaque, le meilleur de `SEED_CANDIDATES` tirages (plus loin des germes déjà posés).
fn pick_seeds(grid: &Grid, seed: u64, count: usize) -> Vec<usize> {
    let mut chosen: Vec<usize> = Vec::with_capacity(count);
    for k in 0..count {
        let candidates = (0..SEED_CANDIDATES).map(|j| {
            let index = k as u32 * SEED_CANDIDATES + j;
            let cell = unit_f64(seed, STEP_ID, index, STREAM_SEED) * grid.cell_count() as f64;
            cell as usize
        });
        let best = candidates
            .filter(|cell| !chosen.contains(cell))
            .max_by(|&a, &b| spread(grid, &chosen, a).total_cmp(&spread(grid, &chosen, b)));
        chosen.push(best.unwrap_or_else(|| first_free(grid, &chosen)));
    }
    chosen
}

/// Distance angulaire minimale (1 − cos) d'une cellule aux germes déjà posés.
fn spread(grid: &Grid, chosen: &[usize], cell: usize) -> f64 {
    let centers = grid.centers();
    chosen
        .iter()
        .map(|&s| 1.0 - dot(centers[cell], centers[s]))
        .fold(f64::INFINITY, f64::min)
}

fn first_free(grid: &Grid, chosen: &[usize]) -> usize {
    (0..grid.cell_count()).find(|c| !chosen.contains(c)).unwrap_or(0)
}

/// Type de croûte par plaque : plaques continentales tirées au hasard (ordre aléatoire) jusqu'à couvrir
/// `CONTINENTAL_AREA_TARGET` de la surface. Au moins une plaque de chaque type.
fn assign_types(grid: &Grid, seed: u64, plate_of: &[u32], count: usize) -> Vec<bool> {
    let mut area = vec![0.0_f64; count];
    for (cell, &p) in plate_of.iter().enumerate() {
        area[p as usize] += grid.areas_m2()[cell];
    }
    let total: f64 = area.iter().sum();
    let mut order: Vec<usize> = (0..count).collect();
    order.sort_by(|&a, &b| type_key(seed, a).total_cmp(&type_key(seed, b)).then(a.cmp(&b)));
    let mut continental = vec![false; count];
    let mut covered = 0.0;
    for &p in &order {
        if covered >= CONTINENTAL_AREA_TARGET * total {
            break;
        }
        continental[p] = true;
        covered += area[p];
    }
    if let Some(&last) = order.last().filter(|_| continental.iter().all(|&c| c)) {
        continental[last] = false;
    }
    continental
}

fn type_key(seed: u64, plate: usize) -> f64 {
    unit_f64(seed, STEP_ID, plate as u32, STREAM_TYPE)
}
