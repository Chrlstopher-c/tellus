//! G6 (relief) : fBm sur les continents, plus des crêtes d'orogène là où le champ tectonique est positif.

use super::g2_motion::Plate;
use super::noise::{fbm, ridged};
use crate::shared::grid::Grid;
use crate::shared::rng::hash;

pub(super) const STEP_ID: u64 = 6;
const FBM_AMPLITUDE_M: f64 = 1_000.0;
const RIDGE_AMPLITUDE_M: f64 = 1_500.0;
const OROGEN_SATURATION_M: f64 = 3_000.0;
const FREQ: f64 = 1.5;
const OCTAVES: u32 = 6;
const STREAM_FBM: u32 = 1;
const STREAM_RIDGE: u32 = 2;

/// Ajoute le relief aux cellules continentales (en place). `tectonic` sert de masque d'orogène.
pub(super) fn add_relief(
    grid: &Grid,
    seed: u64,
    plate_of: &[u32],
    plates: &[Plate],
    tectonic: &[f64],
    heights: &mut [f64],
) {
    let fbm_seed = hash(seed, STEP_ID, 0, STREAM_FBM);
    let ridge_seed = hash(seed, STEP_ID, 0, STREAM_RIDGE);
    for (cell, height) in heights.iter_mut().enumerate() {
        if !plates[plate_of[cell] as usize].continental {
            continue;
        }
        let x = grid.centers()[cell];
        let orogen = (tectonic[cell] / OROGEN_SATURATION_M).clamp(0.0, 1.0);
        *height += FBM_AMPLITUDE_M * fbm(fbm_seed, x, FREQ, OCTAVES)
            + RIDGE_AMPLITUDE_M * orogen * ridged(ridge_seed, x, FREQ * 2.0, OCTAVES);
    }
}
