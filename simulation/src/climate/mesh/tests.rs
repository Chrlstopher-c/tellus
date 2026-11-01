use super::Mesh;
use crate::shared::grid::Grid;
use crate::shared::units::R_EARTH_M;
use std::f64::consts::PI;

fn mesh(level: u8) -> Mesh {
    Mesh::new(&Grid::new(level, R_EARTH_M)).expect("maillage géodésique")
}

#[test]
fn areas_tile_the_unit_sphere() {
    let mesh = mesh(5);
    let sum: f64 = mesh.area_sr().iter().sum();
    assert!((sum - 4.0 * PI).abs() < 1e-9, "somme des aires {sum}");
}

#[test]
fn dual_weights_are_positive_and_symmetric() {
    let mesh = mesh(5);
    for i in 0..mesh.cell_count() {
        for (j, w) in mesh.edges(i) {
            assert!(w.is_finite() && w > 0.0, "poids {w} sur ({i},{j})");
            let back = mesh
                .edges(j)
                .find(|&(k, _)| k == i)
                .map(|(_, w)| w)
                .expect("arête réciproque absente");
            assert!((w - back).abs() <= 1e-12 * w, "({i},{j}) : {w} ≠ {back}");
        }
    }
}

#[test]
fn laplacian_of_constant_is_zero() {
    let mesh = mesh(5);
    let ones = vec![1.0; mesh.cell_count()];
    let mut out = vec![0.0; mesh.cell_count()];
    mesh.laplacian(&ones, &mut out);
    assert!(out.iter().all(|v| v.abs() < 1e-12));
}

#[test]
fn laplacian_conserves_area_weighted_total() {
    let mesh = mesh(5);
    let field: Vec<f64> = (0..mesh.cell_count())
        .map(|i| ((i * 7919) % 1000) as f64 / 1000.0)
        .collect();
    let mut out = vec![0.0; mesh.cell_count()];
    mesh.laplacian(&field, &mut out);
    let total: f64 = mesh.area_sr().iter().zip(&out).map(|(a, v)| a * v).sum();
    let scale: f64 = mesh.area_sr().iter().zip(&out).map(|(a, v)| a * v.abs()).sum();
    assert!(total.abs() <= 1e-12 * scale, "total {total} pour échelle {scale}");
}

#[test]
fn laplacian_of_first_harmonic_is_minus_two() {
    let mesh = mesh(6);
    let field: Vec<f64> = (0..mesh.cell_count()).map(|i| mesh.sin_lat(i)).collect();
    let mut out = vec![0.0; mesh.cell_count()];
    mesh.laplacian(&field, &mut out);
    let worst = (0..mesh.cell_count())
        .filter(|&i| field[i].abs() > 0.2)
        .map(|i| (out[i] / field[i] + 2.0).abs())
        .fold(0.0_f64, f64::max);
    assert!(worst < 0.1, "écart maximal à −2 : {worst}");
}
