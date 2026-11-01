//! Grille géodésique icosaédrique : cellules = sommets primaux, simulation sur les duaux (hexagones + 12 pentagones).

mod dual;
mod geom;
mod icosphere;
mod lloyd;
mod lookup;
mod order;
mod topology;
mod walk;

#[cfg(test)]
mod tests;

pub use lookup::{texel_direction, TexelLookup};

use geom::Vec3;
use topology::Rings;

const LLOYD_ITERATIONS: u32 = 3;

/// Grille de niveau `L` : `N = 10·4^L + 2` cellules, voisinage CSR, aires de Voronoï exactes en m².
pub struct Grid {
    level: u8,
    radius_m: f64,
    centers: Vec<Vec3>,
    triangles: Vec<[u32; 3]>,
    rings: Rings,
    neighbors: Vec<u32>,
    areas_m2: Vec<f64>,
}

impl Grid {
    pub fn new(level: u8, radius_m: f64) -> Self {
        let primal = icosphere::subdivide(level);
        let order = order::morton_order(&primal.positions);
        let (mut centers, triangles) = order::apply(&order, primal.positions, primal.triangles);
        let rings = topology::build_rings(centers.len(), &triangles);
        lloyd::relax(&mut centers, &triangles, &rings, LLOYD_ITERATIONS);
        let areas_m2 = dual::cell_areas(&centers, &triangles, &rings, radius_m);
        let neighbors = topology::neighbors(&rings, &triangles);
        Self {
            level,
            radius_m,
            centers,
            triangles,
            rings,
            neighbors,
            areas_m2,
        }
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn radius_m(&self) -> f64 {
        self.radius_m
    }

    pub fn cell_count(&self) -> usize {
        self.centers.len()
    }

    pub fn centers(&self) -> &[Vec3] {
        &self.centers
    }

    /// Triangles primaux (cellules ordonnées) : base de l'interpolation et du rendu.
    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.triangles
    }

    pub fn neighbor_offsets(&self) -> &[u32] {
        &self.rings.offsets
    }

    pub fn neighbor_indices(&self) -> &[u32] {
        &self.neighbors
    }

    pub fn neighbors(&self, cell: usize) -> &[u32] {
        let range = self.rings.offsets[cell] as usize..self.rings.offsets[cell + 1] as usize;
        &self.neighbors[range]
    }

    pub fn areas_m2(&self) -> &[f64] {
        &self.areas_m2
    }
}
