//! Localisation d'une direction : marche de visibilité sur la triangulation, repli en force brute.

use super::geom::{det, Vec3};

pub struct Locator<'a> {
    centers: &'a [Vec3],
    triangles: &'a [[u32; 3]],
    adjacency: Vec<[u32; 3]>,
    max_steps: usize,
}

impl<'a> Locator<'a> {
    pub fn new(centers: &'a [Vec3], triangles: &'a [[u32; 3]], adjacency: Vec<[u32; 3]>) -> Self {
        let max_steps = 4 * (triangles.len() as f64).sqrt() as usize + 64;
        Self {
            centers,
            triangles,
            adjacency,
            max_steps,
        }
    }

    /// Renvoie (triangle, barycentriques, recours à la force brute).
    pub fn locate(&self, hint: u32, dir: Vec3) -> (u32, [f64; 3], bool) {
        match self.walk(hint, dir) {
            Some((tri, lambda)) => (tri, lambda, false),
            None => {
                let (tri, lambda) = self.brute_force(dir);
                (tri, lambda, true)
            }
        }
    }

    fn walk(&self, start: u32, dir: Vec3) -> Option<(u32, [f64; 3])> {
        let mut tri = start;
        for _ in 0..self.max_steps {
            let lambda = self.barycentric(tri, dir);
            let slot = argmin(&lambda);
            if lambda[slot] >= 0.0 {
                return Some((tri, lambda));
            }
            tri = self.adjacency[tri as usize][slot];
        }
        None
    }

    fn brute_force(&self, dir: Vec3) -> (u32, [f64; 3]) {
        let mut best = (0_u32, [f64::NEG_INFINITY; 3]);
        let mut best_min = f64::NEG_INFINITY;
        for tri in 0..self.triangles.len() {
            let lambda = self.barycentric(tri as u32, dir);
            let worst = lambda.iter().copied().fold(f64::INFINITY, f64::min);
            if worst > best_min {
                best_min = worst;
                best = (tri as u32, lambda);
            }
        }
        best
    }

    /// Coordonnées dans le cône du triangle, normalisées : exactes pour un triangle sphérique (cône convexe).
    fn barycentric(&self, tri: u32, dir: Vec3) -> [f64; 3] {
        let [a, b, c] = self.triangles[tri as usize].map(|v| self.centers[v as usize]);
        let d = det(a, b, c);
        let w = [det(dir, b, c) / d, det(a, dir, c) / d, det(a, b, dir) / d];
        let sum: f64 = w.iter().sum();
        if sum > 0.0 {
            w.map(|x| x / sum)
        } else {
            [f64::NEG_INFINITY; 3]
        }
    }
}

fn argmin(values: &[f64; 3]) -> usize {
    (0..3).min_by(|&i, &j| values[i].total_cmp(&values[j])).unwrap_or(0)
}
