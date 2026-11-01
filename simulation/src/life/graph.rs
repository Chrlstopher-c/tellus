//! Voisinage CSR des cellules, copié de la grille : base de la dispersion, de la connectivité et de la spéciation.

use crate::shared::grid::Grid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    offsets: Vec<u32>,
    indices: Vec<u32>,
}

impl Graph {
    pub fn new(offsets: Vec<u32>, indices: Vec<u32>) -> Self {
        Self { offsets, indices }
    }

    pub fn from_grid(grid: &Grid) -> Self {
        Self::new(grid.neighbor_offsets().to_vec(), grid.neighbor_indices().to_vec())
    }

    pub fn cell_count(&self) -> usize {
        self.offsets.len() - 1
    }

    pub fn neighbors(&self, cell: usize) -> &[u32] {
        let range = self.offsets[cell] as usize..self.offsets[cell + 1] as usize;
        &self.indices[range]
    }
}
