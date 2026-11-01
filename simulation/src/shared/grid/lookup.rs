//! Table texel → (3 cellules du triangle primal, 2 barycentriques) pour le bake équirectangulaire.
//! Convention : u = lon, v = lat, dir = (cos lat·sin lon, sin lat, cos lat·cos lon).

use std::f64::consts::PI;

use super::geom::Vec3;
use super::topology::triangle_adjacency;
use super::walk::Locator;
use super::Grid;

/// Pour chaque texel (ligne-major) : sommets du triangle et λ₀, λ₁ (λ₂ = 1 − λ₀ − λ₁).
pub struct TexelLookup {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<[u32; 3]>,
    pub weights: Vec<[f32; 2]>,
    /// Nombre de texels résolus par force brute faute de marche réussie (attendu : 0).
    pub fallbacks: u32,
}

impl Grid {
    pub fn texel_lookup(&self, width: u32, height: u32) -> TexelLookup {
        let adjacency = triangle_adjacency(&self.rings, &self.triangles);
        let locator = Locator::new(&self.centers, &self.triangles, adjacency);
        let total = (width * height) as usize;
        let mut table = TexelLookup {
            width,
            height,
            cells: Vec::with_capacity(total),
            weights: Vec::with_capacity(total),
            fallbacks: 0,
        };
        let mut hint = 0_u32;
        for y in 0..height {
            for x in 0..width {
                let (tri, lambda, fallback) = locator.locate(hint, texel_direction(x, y, width, height));
                hint = tri;
                table.fallbacks += u32::from(fallback);
                table.cells.push(self.triangles[tri as usize]);
                table.weights.push([lambda[0] as f32, lambda[1] as f32]);
            }
        }
        table
    }
}

/// Direction unitaire du centre du texel (x, y) selon la convention de la maquette.
pub fn texel_direction(x: u32, y: u32, width: u32, height: u32) -> Vec3 {
    let u = (f64::from(x) + 0.5) / f64::from(width);
    let v = (f64::from(y) + 0.5) / f64::from(height);
    let lon = (u - 0.5) * 2.0 * PI;
    let lat = (v - 0.5) * PI;
    [lat.cos() * lon.sin(), lat.sin(), lat.cos() * lon.cos()]
}
