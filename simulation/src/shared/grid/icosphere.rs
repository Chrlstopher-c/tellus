//! Icosaèdre subdivisé en `freq²` triangles par face ; les sommets sont les centres des cellules.

use super::geom::{add, det, normalize, scale, Vec3};

const PHI: f64 = std::f64::consts::GOLDEN_RATIO;

const WARP_C: f64 = 1.2;

/// Réparamétrage symétrique t ↦ g(t) (g(1−t) = 1−g(t)) qui compense la distorsion gnomonique.
fn warp(t: f64) -> f64 {
    let c = WARP_C;
    0.5 + (c * (t - 0.5)).tan() / (2.0 * (c / 2.0).tan())
}

const CORNERS: [Vec3; 12] = [
    [-1.0, PHI, 0.0],
    [1.0, PHI, 0.0],
    [-1.0, -PHI, 0.0],
    [1.0, -PHI, 0.0],
    [0.0, -1.0, PHI],
    [0.0, 1.0, PHI],
    [0.0, -1.0, -PHI],
    [0.0, 1.0, -PHI],
    [PHI, 0.0, -1.0],
    [PHI, 0.0, 1.0],
    [-PHI, 0.0, -1.0],
    [-PHI, 0.0, 1.0],
];

const FACES: [[usize; 3]; 20] = [
    [0, 11, 5],
    [0, 5, 1],
    [0, 1, 7],
    [0, 7, 10],
    [0, 10, 11],
    [1, 5, 9],
    [5, 11, 4],
    [11, 10, 2],
    [10, 7, 6],
    [7, 1, 8],
    [3, 9, 4],
    [3, 4, 2],
    [3, 2, 6],
    [3, 6, 8],
    [3, 8, 9],
    [4, 9, 5],
    [2, 4, 11],
    [6, 2, 10],
    [8, 6, 7],
    [9, 8, 1],
];

/// Sommets et triangles primaux, triangles orientés CCW vu de l'extérieur.
pub struct Primal {
    pub positions: Vec<Vec3>,
    pub triangles: Vec<[u32; 3]>,
}

/// Triangulation de niveau `level` : `10·4^level + 2` sommets, `20·4^level` triangles.
pub fn subdivide(level: u8) -> Primal {
    let freq = 1_usize << level;
    let corners: Vec<Vec3> = CORNERS.iter().map(|&c| normalize(c)).collect();
    let mut lattice = Lattice::new(&corners, freq);
    let mut triangles = Vec::with_capacity(20 * freq * freq);
    for face in FACES {
        let oriented = orient(face, &corners);
        let table = lattice.face_table(oriented);
        push_triangles(&table, freq, &mut triangles);
    }
    Primal {
        positions: lattice.positions,
        triangles,
    }
}

fn orient(face: [usize; 3], corners: &[Vec3]) -> [usize; 3] {
    let [a, b, c] = face;
    if det(corners[a], corners[b], corners[c]) < 0.0 {
        [a, c, b]
    } else {
        face
    }
}

/// Table des points partagés : sommets d'icosaèdre, points d'arêtes (une fois par arête), points intérieurs.
struct Lattice<'a> {
    corners: &'a [Vec3],
    freq: usize,
    edge_base: [[usize; 12]; 12],
    positions: Vec<Vec3>,
}

impl<'a> Lattice<'a> {
    fn new(corners: &'a [Vec3], freq: usize) -> Self {
        let mut lattice = Self {
            corners,
            freq,
            edge_base: [[0; 12]; 12],
            positions: corners.to_vec(),
        };
        for face in FACES {
            let pairs = [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])];
            for (a, b) in pairs {
                let (p, q) = (a.min(b), a.max(b));
                if lattice.edge_base[p][q] == 0 {
                    lattice.add_edge_points(p, q);
                }
            }
        }
        lattice
    }

    fn add_edge_points(&mut self, p: usize, q: usize) {
        self.edge_base[p][q] = self.positions.len();
        let (a, b) = (self.corners[p], self.corners[q]);
        for k in 1..self.freq {
            let t = warp(k as f64 / self.freq as f64);
            let point = add(scale(a, 1.0 - t), scale(b, t));
            self.positions.push(normalize(point));
        }
    }

    /// Index du point situé à `k` pas de `p` vers `q` sur l'arête (1 ≤ k ≤ freq − 1).
    fn edge_point(&self, p: usize, q: usize, k: usize) -> u32 {
        let index = if p < q {
            self.edge_base[p][q] + k - 1
        } else {
            self.edge_base[q][p] + (self.freq - k) - 1
        };
        index as u32
    }

    fn lattice_point(&mut self, [a, b, c]: [usize; 3], i: usize, j: usize) -> u32 {
        let f = self.freq;
        match (i, j) {
            (0, 0) => a as u32,
            (_, 0) if i == f => b as u32,
            (0, _) if j == f => c as u32,
            (_, 0) => self.edge_point(a, b, i),
            (0, _) => self.edge_point(a, c, j),
            _ if i + j == f => self.edge_point(b, c, j),
            _ => self.push_interior([a, b, c], i, j),
        }
    }

    fn push_interior(&mut self, [a, b, c]: [usize; 3], i: usize, j: usize) -> u32 {
        let f = self.freq as f64;
        let w = [(self.freq - i - j) as f64 / f, i as f64 / f, j as f64 / f].map(warp);
        let sum = w[0] + w[1] + w[2];
        let point = add(
            add(scale(self.corners[a], w[0] / sum), scale(self.corners[b], w[1] / sum)),
            scale(self.corners[c], w[2] / sum),
        );
        self.positions.push(normalize(point));
        (self.positions.len() - 1) as u32
    }

    /// Indices des points de la face, rangés en `[i][j]` (i + j ≤ freq) dans une table plate.
    fn face_table(&mut self, corners: [usize; 3]) -> Vec<u32> {
        let n = self.freq + 1;
        let mut table = vec![0_u32; n * n];
        for i in 0..=self.freq {
            for j in 0..=(self.freq - i) {
                table[i * n + j] = self.lattice_point(corners, i, j);
            }
        }
        table
    }
}

fn push_triangles(table: &[u32], freq: usize, out: &mut Vec<[u32; 3]>) {
    let n = freq + 1;
    let at = |i: usize, j: usize| table[i * n + j];
    for i in 0..freq {
        for j in 0..(freq - i) {
            out.push([at(i, j), at(i + 1, j), at(i, j + 1)]);
            if j + 1 < freq - i {
                out.push([at(i + 1, j), at(i + 1, j + 1), at(i, j + 1)]);
            }
        }
    }
}
