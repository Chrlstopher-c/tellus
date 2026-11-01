//! Relaxation de Lloyd sur les duaux : chaque centre glisse vers le centroïde de son polygone.
//! Le polygone de Voronoï est formé par les circoncentres sphériques des triangles incidents.

use super::dual::dual_polygon;
use super::geom::{add, cross, norm, normalize, scale, sub, Vec3};
use super::topology::Rings;

pub fn relax(centers: &mut [Vec3], tris: &[[u32; 3]], rings: &Rings, iterations: u32) {
    for _ in 0..iterations {
        let next: Vec<Vec3> = (0..centers.len())
            .map(|v| relaxed_center(v, centers, tris, rings))
            .collect();
        centers.copy_from_slice(&next);
    }
}

fn relaxed_center(v: usize, centers: &[Vec3], tris: &[[u32; 3]], rings: &Rings) -> Vec3 {
    let g = centers[v];
    let corners = dual_polygon(v, rings, tris, centers);
    let mut weighted = [0.0_f64; 3];
    let mut total = 0.0;
    for (a, b) in corners.iter().zip(corners.iter().cycle().skip(1)) {
        let area = 0.5 * norm(cross(sub(*a, g), sub(*b, g)));
        let centroid = scale(add(add(g, *a), *b), 1.0 / 3.0);
        weighted = add(weighted, scale(centroid, area));
        total += area;
    }
    normalize(scale(weighted, 1.0 / total))
}
