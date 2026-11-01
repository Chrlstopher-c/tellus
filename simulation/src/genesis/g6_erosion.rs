//! G6 (érosion) : érosion fluviale explicite (∝ √(aire drainée) × dénivelé) et thermique au-delà du talus.
//! Le niveau de base est la mer : un cours d'eau ne creuse pas sous le niveau de la mer. Le matériau érodé est
//! exporté et réparti sur l'océan par aire : aucune cellule continentale ne remonte, et la masse est conservée
//! (Σ A·Δh = 0).

use super::graph::arc_m;
use crate::shared::grid::Grid;

/// Volumes (m³) de matériau retiré à la terre et déposé sur l'océan, pour le contrôle de conservation.
pub(super) struct ErosionReport {
    pub(super) removed_m3: f64,
    pub(super) deposited_m3: f64,
}

const ITERATIONS: u32 = 60;
const FLUVIAL_COEFF: f64 = 0.01;
const MAX_FLUVIAL_FRACTION: f64 = 0.5;
const TALUS_SLOPE: f64 = 0.7;
const THERMAL_FRACTION: f64 = 0.5;

/// Érode `heights` en place (cellules de `land` seulement) au-dessus du niveau de base `base_level_m`.
pub(super) fn erode(grid: &Grid, land: &[bool], heights: &mut [f64], base_level_m: f64) -> ErosionReport {
    let areas = grid.areas_m2();
    let before = heights.to_vec();
    let mut removed_m3 = 0.0;
    for _ in 0..ITERATIONS {
        let removal = removal_step(grid, land, heights, base_level_m);
        for (cell, &amount) in removal.iter().enumerate() {
            heights[cell] -= amount;
            removed_m3 += amount * areas[cell];
        }
    }
    let ocean_area: f64 = (0..heights.len()).filter(|&c| !land[c]).map(|c| areas[c]).sum();
    let raise = removed_m3 / ocean_area;
    for (cell, height) in heights.iter_mut().enumerate() {
        if !land[cell] {
            *height += raise;
        }
    }
    let deposited_m3 = (0..heights.len())
        .filter(|&c| !land[c])
        .map(|c| areas[c] * (heights[c] - before[c]))
        .sum();
    ErosionReport {
        removed_m3,
        deposited_m3,
    }
}

/// Retrait (m) de chaque cellule continentale pendant un pas : fluvial + thermique, jamais plus que le dénivelé
/// jusqu'au receveur (ou jusqu'au niveau de base si le receveur est sous la mer).
fn removal_step(grid: &Grid, land: &[bool], heights: &[f64], base_level_m: f64) -> Vec<f64> {
    let receivers = receivers(grid, heights);
    let drained = accumulate(grid, land, heights, &receivers);
    let mean_area = grid.areas_m2().iter().sum::<f64>() / grid.cell_count() as f64;
    (0..grid.cell_count())
        .map(|cell| {
            let receiver = receivers[cell];
            if !land[cell] || receiver == cell {
                return 0.0;
            }
            let drop = heights[cell] - heights[receiver].max(base_level_m);
            if drop <= 0.0 {
                return 0.0;
            }
            let fluvial = (FLUVIAL_COEFF * (drained[cell] / mean_area).sqrt() * drop).min(MAX_FLUVIAL_FRACTION * drop);
            let excess = drop - TALUS_SLOPE * arc_m(grid, cell, receiver);
            fluvial + THERMAL_FRACTION * excess.max(0.0)
        })
        .collect()
}

/// Receveur de chaque cellule : voisin le plus bas s'il est plus bas qu'elle, sinon elle-même (cuvette).
fn receivers(grid: &Grid, heights: &[f64]) -> Vec<usize> {
    (0..grid.cell_count())
        .map(|cell| {
            let lowest = grid
                .neighbors(cell)
                .iter()
                .map(|&nb| nb as usize)
                .min_by(|&a, &b| heights[a].total_cmp(&heights[b]).then(a.cmp(&b)));
            match lowest {
                Some(low) if heights[low] < heights[cell] => low,
                _ => cell,
            }
        })
        .collect()
}

/// Aire drainée (m²) par cellule continentale, du haut vers le bas : chaque cellule transmet son aire à son receveur.
fn accumulate(grid: &Grid, land: &[bool], heights: &[f64], receivers: &[usize]) -> Vec<f64> {
    let mut order: Vec<usize> = (0..grid.cell_count()).filter(|&c| land[c]).collect();
    order.sort_unstable_by(|&a, &b| heights[b].total_cmp(&heights[a]).then(a.cmp(&b)));
    let mut drained = grid.areas_m2().to_vec();
    for &cell in &order {
        let receiver = receivers[cell];
        if receiver != cell && land[receiver] {
            let carried = drained[cell];
            drained[receiver] += carried;
        }
    }
    drained
}
