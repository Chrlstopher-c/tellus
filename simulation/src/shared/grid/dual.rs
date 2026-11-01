//! Cellules duales : polygone de Voronoï (circoncentres des triangles incidents) et aire exacte par éventail.

use super::geom::{circumcenter, solid_angle, Vec3};
use super::topology::Rings;

/// Sommets du polygone de Voronoï du sommet `vertex`, dans l'ordre CCW.
pub fn dual_polygon(vertex: usize, rings: &Rings, tris: &[[u32; 3]], centers: &[Vec3]) -> Vec<Vec3> {
    rings
        .of(vertex)
        .iter()
        .map(|&t| {
            let [a, b, c] = tris[t as usize].map(|v| centers[v as usize]);
            circumcenter(a, b, c)
        })
        .collect()
}

/// Aire de chaque cellule en m² : éventail de triangles sphériques (centre, circoncentres consécutifs).
/// Les cellules tapissent la sphère : leur somme vaut 4πR² au point flottant près.
pub fn cell_areas(centers: &[Vec3], tris: &[[u32; 3]], rings: &Rings, radius_m: f64) -> Vec<f64> {
    let unit = radius_m * radius_m;
    (0..centers.len())
        .map(|v| {
            let g = centers[v];
            let polygon = dual_polygon(v, rings, tris, centers);
            let steradians: f64 = polygon
                .iter()
                .zip(polygon.iter().cycle().skip(1))
                .map(|(&a, &b)| solid_angle(g, a, b))
                .sum();
            steradians * unit
        })
        .collect()
}
