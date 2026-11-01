//! Orographie (climat.md §6) : gradient d'altitude sur le maillage duale (formule de Gauss), soulèvement de l'air au
//! vent `max(0, V·∇z)` et taux de condensation orographique `soulèvement / H_w`. L'ombre pluviométrique n'est pas
//! posée ici : elle naît du transport de vapeur qui se vide sur le versant au vent.

use super::mesh::Mesh;

/// Hauteur d'échelle de la condensation orographique, m (climat.md §6 : H_w ≈ 2,5 km).
pub const OROGRAPHIC_SCALE_HEIGHT_M: f64 = 2_500.0;

/// Gradient d'altitude de chaque cellule, m rad⁻¹ dans le repère unitaire : `∇z = (1/A) Σ_k ℓ_k n_k (z_j − z_i)/2`.
pub fn gradient(mesh: &Mesh, elevation_m: &[f64]) -> Vec<[f64; 3]> {
    let offsets = mesh.edge_offsets();
    let neighbours = mesh.neighbor_indices();
    let flows = mesh.edge_flow();
    let areas = mesh.area_sr();
    (0..mesh.cell_count())
        .map(|cell| {
            let mut grad = [0.0; 3];
            for k in offsets[cell] as usize..offsets[cell + 1] as usize {
                let other = neighbours[k] as usize;
                let weight = 0.5 * (elevation_m[other] - elevation_m[cell]) / areas[cell];
                for (axis, slot) in grad.iter_mut().enumerate() {
                    *slot += weight * flows[k][axis];
                }
            }
            grad
        })
        .collect()
}

/// Taux de condensation orographique par cellule, s⁻¹ : `max(0, V·∇z) / H_w`, `V` tangent (m s⁻¹, repère 3D).
pub fn uplift_rate(gradients: &[[f64; 3]], winds: &[[f64; 3]], radius_m: f64) -> Vec<f64> {
    gradients
        .iter()
        .zip(winds)
        .map(|(grad, wind)| (dot(grad, wind) / radius_m).max(0.0) / OROGRAPHIC_SCALE_HEIGHT_M)
        .collect()
}

fn dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::grid::Grid;
    use crate::shared::units::R_EARTH_M;
    use std::f64::consts::PI;

    /// Crête méridienne de 2 000 m centrée sur `lon0` (radians), largeur ~ 5°.
    fn ridge(grid: &Grid, lon0: f64) -> Vec<f64> {
        grid.centers()
            .iter()
            .map(|c| {
                let dlon = (c[2].atan2(c[0]) - lon0 + PI).rem_euclid(2.0 * PI) - PI;
                2_000.0 * (-(dlon / 5_f64.to_radians()).powi(2)).exp()
            })
            .collect()
    }

    /// Vent uniforme vers l'ouest, tangent à la sphère : opposé du vecteur est `(−z, 0, x)`.
    fn westward(grid: &Grid) -> Vec<[f64; 3]> {
        grid.centers().iter().map(|c| [c[2], 0.0, -c[0]]).collect()
    }

    fn mean_lift_in_band(grid: &Grid, lift: &[f64], lon_min_deg: f64, lon_max_deg: f64) -> f64 {
        let (sum, count) = grid
            .centers()
            .iter()
            .zip(lift)
            .filter(|(c, _)| {
                let lon = c[2].atan2(c[0]).to_degrees();
                (lon_min_deg..lon_max_deg).contains(&lon) && c[1].abs() < 0.2
            })
            .fold((0.0, 0), |(s, n), (_, l)| (s + l, n + 1));
        sum / f64::from(count.max(1))
    }

    #[test]
    fn gradient_of_a_linear_slope_is_its_tangent_projection() {
        let grid = Grid::new(4, R_EARTH_M);
        let mesh = Mesh::new(&grid).expect("maillage");
        let elevation: Vec<f64> = grid.centers().iter().map(|c| 1_000.0 * c[0]).collect();
        let grads = gradient(&mesh, &elevation);
        let c = grid.centers()[40];
        let expected = [1_000.0 * (1.0 - c[0] * c[0]), -1_000.0 * c[0] * c[1], -1_000.0 * c[0] * c[2]];
        let err = (0..3).map(|a| (grads[40][a] - expected[a]).abs()).fold(0.0, f64::max);
        assert!(err < 150.0, "gradient erroné : {:?} vs {expected:?}", grads[40]);
    }

    #[test]
    fn uplift_is_positive_on_the_windward_flank_only() {
        let grid = Grid::new(5, R_EARTH_M);
        let mesh = Mesh::new(&grid).expect("maillage");
        let grads = gradient(&mesh, &ridge(&grid, 0.0));
        let lift = uplift_rate(&grads, &westward(&grid), R_EARTH_M);
        let east = mean_lift_in_band(&grid, &lift, 1.0, 4.0);
        let west = mean_lift_in_band(&grid, &lift, -4.0, -1.0);
        assert!(east > 0.0, "flanc est (au vent d'un vent d'ouest) : {east}");
        assert!(west < 0.5 * east, "flanc ouest sous le vent : {west} vs {east}");
    }
}
