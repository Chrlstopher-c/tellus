//! Table de rendu : pour chaque texel équirectangulaire, 3 cellules (RGB, A = 0) et 2 poids (RG), en flottants.

use crate::shared::grid::Grid;

pub const LOOKUP_WIDTH: u32 = 2048;
pub const LOOKUP_HEIGHT: u32 = 1024;

pub struct Lookup {
    pub ids: Vec<f32>,
    pub weights: Vec<f32>,
}

/// `Grid::texel_lookup` au format attendu par le rendu : ids RGBA (4 par texel), poids RG (2 par texel).
pub fn build(grid: &Grid) -> Lookup {
    let table = grid.texel_lookup(LOOKUP_WIDTH, LOOKUP_HEIGHT);
    let ids = table
        .cells
        .iter()
        .flat_map(|cell| [cell[0] as f32, cell[1] as f32, cell[2] as f32, 0.0])
        .collect();
    let weights = table.weights.iter().flat_map(|w| [w[0], w[1]]).collect();
    Lookup { ids, weights }
}
