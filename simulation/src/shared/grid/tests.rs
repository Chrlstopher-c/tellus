use super::lookup::TexelLookup;
use super::*;
use crate::shared::rng::unit_f64;
use crate::shared::units::R_EARTH_M;
use std::time::Instant;

fn norm(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[test]
fn grid_cell_count() {
    for level in [4_u8, 6, 7] {
        let grid = Grid::new(level, R_EARTH_M);
        let expected = 10 * 4_usize.pow(u32::from(level)) + 2;
        assert_eq!(grid.cell_count(), expected, "L{level}");
        assert_eq!(grid.centers().len(), expected);
        assert!(grid.centers().iter().all(|c| (norm(*c) - 1.0).abs() < 1e-12));
    }
}

#[test]
fn grid_valence() {
    let grid = Grid::new(6, R_EARTH_M);
    let mut pentagons = 0;
    for cell in 0..grid.cell_count() {
        match grid.neighbors(cell).len() {
            5 => pentagons += 1,
            6 => {}
            other => panic!("cellule {cell} de valence {other}"),
        }
    }
    assert_eq!(pentagons, 12);
}

#[test]
fn grid_neighbors_symmetric() {
    let grid = Grid::new(5, R_EARTH_M);
    for cell in 0..grid.cell_count() {
        for &other in grid.neighbors(cell) {
            assert!(grid.neighbors(other as usize).contains(&(cell as u32)));
            assert_ne!(other as usize, cell);
        }
    }
    assert_eq!(grid.neighbor_offsets().len(), grid.cell_count() + 1);
    assert_eq!(
        *grid.neighbor_offsets().last().unwrap() as usize,
        grid.neighbor_indices().len()
    );
}

#[test]
fn grid_area_sum() {
    let grid = Grid::new(6, R_EARTH_M);
    let sum: f64 = grid.areas_m2().iter().sum();
    let expected = 4.0 * std::f64::consts::PI * R_EARTH_M * R_EARTH_M;
    assert!(
        ((sum - expected) / expected).abs() < 1e-9,
        "relatif {}",
        (sum - expected) / expected
    );
}

#[test]
fn grid_area_uniformity() {
    let grid = Grid::new(6, R_EARTH_M);
    let areas = grid.areas_m2();
    let max = areas.iter().copied().fold(f64::MIN, f64::max);
    let min = areas.iter().copied().fold(f64::MAX, f64::min);
    assert!(max / min < 1.25, "max/min = {}", max / min);
}

#[test]
fn grid_triangles_ccw_and_closed() {
    let grid = Grid::new(4, R_EARTH_M);
    let c = grid.centers();
    for t in grid.triangles() {
        let (a, b, d) = (c[t[0] as usize], c[t[1] as usize], c[t[2] as usize]);
        let det = dot(
            a,
            [
                b[1] * d[2] - b[2] * d[1],
                b[2] * d[0] - b[0] * d[2],
                b[0] * d[1] - b[1] * d[0],
            ],
        );
        assert!(det > 0.0, "triangle non CCW");
    }
    assert_eq!(grid.triangles().len(), 20 * 16 * 16);
}

#[test]
fn grid_deterministic() {
    let a = Grid::new(5, R_EARTH_M);
    let b = Grid::new(5, R_EARTH_M);
    assert_eq!(a.centers(), b.centers());
    assert_eq!(a.neighbor_indices(), b.neighbor_indices());
    assert_eq!(a.areas_m2(), b.areas_m2());
}

#[test]
fn grid_order_local() {
    let grid = Grid::new(6, R_EARTH_M);
    let mut gaps: Vec<usize> = Vec::new();
    for cell in 0..grid.cell_count() {
        for &other in grid.neighbors(cell) {
            gaps.push(cell.abs_diff(other as usize));
        }
    }
    gaps.sort_unstable();
    let median = gaps[gaps.len() / 2];
    assert!(median < 200, "écart médian d'indices entre voisins : {median}");
}

#[test]
fn texel_lookup_valid() {
    let grid = Grid::new(7, R_EARTH_M);
    let (w, h) = (512_u32, 256_u32);
    let table = grid.texel_lookup(w, h);
    assert_eq!(table.cells.len(), (w * h) as usize);
    assert_eq!(table.fallbacks, 0);
    for (cell, weight) in table.cells.iter().zip(&table.weights) {
        assert!(cell.iter().all(|&c| (c as usize) < grid.cell_count()));
        let (l0, l1) = (f64::from(weight[0]), f64::from(weight[1]));
        let l2 = 1.0 - l0 - l1;
        assert!(l0 >= -1e-6 && l1 >= -1e-6 && l2 >= -1e-6);
        assert!(l0 + l1 <= 1.0 + 1e-6);
    }
    let hits = nearest_agreement(&grid, &table, w, h, 2000);
    assert_eq!(
        hits,
        2000,
        "cellule la plus proche hors du triangle dans {} cas",
        2000 - hits
    );
}

/// Nombre de texels aléatoires dont la cellule la plus proche (force brute) est un sommet du triangle trouvé.
/// Le plus fort poids barycentrique n'est pas toujours le plus proche dans un triangle non équilatéral.
fn nearest_agreement(grid: &Grid, table: &TexelLookup, w: u32, h: u32, samples: u32) -> usize {
    let mut hits = 0;
    for sample in 0..samples {
        let x = (unit_f64(11, 0, sample, 0) * f64::from(w)) as u32 % w;
        let y = (unit_f64(11, 0, sample, 1) * f64::from(h)) as u32 % h;
        let idx = (y * w + x) as usize;
        let dir = super::lookup::texel_direction(x, y, w, h);
        let nearest = (0..grid.cell_count())
            .max_by(|&i, &j| dot(grid.centers()[i], dir).total_cmp(&dot(grid.centers()[j], dir)))
            .unwrap_or(0);
        if table.cells[idx].iter().any(|&c| c as usize == nearest) {
            hits += 1;
        }
    }
    hits
}

#[test]
#[ignore]
fn grid_build_l7_under_3s() {
    let start = Instant::now();
    let grid = Grid::new(7, R_EARTH_M);
    let elapsed = start.elapsed().as_secs_f64();
    eprintln!("construction L7 : {elapsed:.3} s ({} cellules)", grid.cell_count());
    assert_eq!(grid.cell_count(), 163_842);
    assert!(elapsed < 3.0, "L7 en {elapsed:.3} s");
}
