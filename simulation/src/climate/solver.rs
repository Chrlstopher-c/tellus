//! Solveur de l'Euler implicite : gradient conjugué préconditionné Jacobi, départ à chaud (x = T précédente),
//! noyaux fusionnés et parallélisés avec rayon par blocs de taille fixe.
//! Déterminisme : chaque élément est calculé indépendamment ; chaque somme est calculée par bloc de taille fixe,
//! puis les sommes partielles sont réduites dans l'ordre des blocs. Le résultat est donc bit à bit identique
//! quel que soit le nombre de threads.
//! Opérateur : `(A x)_i = a_i c_i x_i + D Σ_j w_ij (x_i − x_j)`, symétrique définie positive.
//! Après le CG, un décalage uniforme absorbe la somme du résidu : `1ᵀ(A x − b) = 0` exactement.

use super::mesh::Mesh;
use rayon::prelude::*;

/// Itérations maximales par pas implicite ; atteindre ce plafond est compté dans le rapport.
pub const CG_MAX_ITER: u32 = 300;
/// Résidu relatif visé (‖b − A x‖ / ‖b‖).
pub const CG_TOLERANCE: f64 = 1e-6;
/// Taille fixe des blocs : fixe le découpage des sommes, donc le résultat.
const BLOCK: usize = 4096;

/// Opérateur implicite `diag(a·c) + D·K` sur un maillage.
pub struct ImplicitOperator<'a> {
    mesh: &'a Mesh,
    mass: Vec<f64>,
    diffusivity: f64,
    diagonal: Vec<f64>,
}

/// Bilan d'une résolution. `energy_error` est le résidu d'énergie laissé par le CG, relatif à Σ|b|, avant correction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolveReport {
    pub iterations: u32,
    pub relative_residual: f64,
    pub energy_error: f64,
    /// Vrai si le résidu visé n'est pas atteint (plafond d'itérations ou rupture numérique).
    pub cap_reached: bool,
}

impl<'a> ImplicitOperator<'a> {
    /// `mass[i]` = `C_i/dt + λ_i` (W m⁻² K⁻¹), `diffusivity` = D (W m⁻² K⁻¹).
    pub fn new(mesh: &'a Mesh, mass: &[f64], diffusivity: f64) -> Self {
        let scaled: Vec<f64> = mass.iter().zip(mesh.area_sr()).map(|(m, a)| m * a).collect();
        let diagonal = (0..mesh.cell_count())
            .map(|i| {
                let coupling: f64 = mesh.edges(i).map(|(_, w)| w).sum();
                scaled[i] + diffusivity * coupling
            })
            .collect();
        Self {
            mesh,
            mass: scaled,
            diffusivity,
            diagonal,
        }
    }

    /// `y = A x`.
    pub fn apply(&self, x: &[f64], y: &mut [f64]) {
        self.apply_dot(x, y);
    }

    /// `y = A x` et renvoie `xᵀ·y` dans la même passe.
    fn apply_dot(&self, x: &[f64], y: &mut [f64]) -> f64 {
        block_map_mut(y, |start, chunk| {
            let mut acc = 0.0;
            for (k, slot) in chunk.iter_mut().enumerate() {
                let i = start + k;
                let coupling: f64 = self.mesh.edges(i).map(|(j, w)| w * (x[i] - x[j])).sum();
                let value = self.mass[i] * x[i] + self.diffusivity * coupling;
                *slot = value;
                acc += x[i] * value;
            }
            acc
        })
    }

    /// `Σ_i a_i c_i` : facteur liant le décalage uniforme à la somme des résidus.
    pub fn mass_sum(&self) -> f64 {
        block_sum(&self.mass)
    }

    fn diagonal(&self) -> &[f64] {
        &self.diagonal
    }
}

/// Résout `A x = b` depuis l'estimation initiale `x` (départ à chaud), au plus `CG_MAX_ITER` itérations.
pub fn solve(op: &ImplicitOperator<'_>, rhs: &[f64], x: &mut [f64]) -> SolveReport {
    solve_with(op, rhs, x, CG_MAX_ITER, CG_TOLERANCE)
}

/// Même résolution avec bornes explicites (tests de convergence).
pub fn solve_with(op: &ImplicitOperator<'_>, rhs: &[f64], x: &mut [f64], max_iter: u32, tol: f64) -> SolveReport {
    let (iterations, relative_residual) = conjugate_gradient(op, rhs, x, max_iter, tol);
    let energy_error = conserve_energy(op, rhs, x);
    SolveReport {
        iterations,
        relative_residual,
        energy_error,
        cap_reached: relative_residual > tol,
    }
}

fn conjugate_gradient(op: &ImplicitOperator<'_>, rhs: &[f64], x: &mut [f64], max_iter: u32, tol: f64) -> (u32, f64) {
    let rhs_norm = dot(rhs, rhs).sqrt();
    if rhs_norm == 0.0 {
        x.iter_mut().for_each(|v| *v = 0.0);
        return (0, 0.0);
    }
    let mut ax = vec![0.0; rhs.len()];
    op.apply(x, &mut ax);
    let mut residual = difference(rhs, &ax);
    let mut relative = dot(&residual, &residual).sqrt() / rhs_norm;
    let mut precond = vec![0.0; rhs.len()];
    let mut rz = precondition(&residual, op.diagonal(), &mut precond);
    let mut direction = precond.clone();
    let mut iterations = 0;
    while relative > tol && iterations < max_iter {
        let curvature = op.apply_dot(&direction, &mut ax);
        if !(curvature > 0.0 && curvature.is_finite()) {
            break;
        }
        let alpha = rz / curvature;
        let residual_sq = update_iterate(x, &mut residual, &direction, &ax, alpha);
        relative = residual_sq.sqrt() / rhs_norm;
        iterations += 1;
        if relative <= tol {
            break;
        }
        let rz_next = precondition(&residual, op.diagonal(), &mut precond);
        let beta = rz_next / rz;
        rz = rz_next;
        advance_direction(&mut direction, &precond, beta);
    }
    (iterations, relative)
}

/// Décalage uniforme `η` tel que `1ᵀ(A (x + η·1) − b) = 0` ; renvoie le résidu relatif avant correction.
fn conserve_energy(op: &ImplicitOperator<'_>, rhs: &[f64], x: &mut [f64]) -> f64 {
    let mut ax = vec![0.0; x.len()];
    op.apply(x, &mut ax);
    let residual = difference(rhs, &ax);
    let residual_sum = block_sum(&residual);
    let scale = block_reduce(rhs, |chunk| chunk.iter().map(|b| b.abs()).sum());
    let shift = residual_sum / op.mass_sum();
    x.par_iter_mut().for_each(|v| *v += shift);
    if scale > 0.0 {
        residual_sum.abs() / scale
    } else {
        0.0
    }
}

/// `x += α p`, `r −= α Ap` dans une seule passe ; renvoie `‖r‖²`.
fn update_iterate(x: &mut [f64], r: &mut [f64], p: &[f64], ap: &[f64], alpha: f64) -> f64 {
    let partial: Vec<f64> = x
        .par_chunks_mut(BLOCK)
        .zip(r.par_chunks_mut(BLOCK))
        .enumerate()
        .map(|(b, (xc, rc))| {
            let start = b * BLOCK;
            let len = xc.len();
            let mut acc = 0.0;
            for (((xv, rv), pv), av) in xc
                .iter_mut()
                .zip(rc.iter_mut())
                .zip(&p[start..start + len])
                .zip(&ap[start..start + len])
            {
                *xv += alpha * pv;
                *rv -= alpha * av;
                acc += *rv * *rv;
            }
            acc
        })
        .collect();
    partial.iter().sum()
}

/// `z = r / d` ; renvoie `rᵀ z`.
fn precondition(residual: &[f64], diagonal: &[f64], out: &mut [f64]) -> f64 {
    block_map_mut(out, |start, chunk| {
        let len = chunk.len();
        let mut acc = 0.0;
        for ((zv, rv), dv) in chunk
            .iter_mut()
            .zip(&residual[start..start + len])
            .zip(&diagonal[start..start + len])
        {
            *zv = rv / dv;
            acc += rv * *zv;
        }
        acc
    })
}

/// `p = z + β p`, élément par élément.
fn advance_direction(direction: &mut [f64], z: &[f64], beta: f64) {
    direction
        .par_chunks_mut(BLOCK)
        .zip(z.par_chunks(BLOCK))
        .for_each(|(pc, zc)| {
            for (pv, zv) in pc.iter_mut().zip(zc) {
                *pv = zv + beta * *pv;
            }
        });
}

fn difference(a: &[f64], b: &[f64]) -> Vec<f64> {
    a.par_iter().zip(b.par_iter()).map(|(p, q)| p - q).collect()
}

/// Produit scalaire à blocs fixes : sommes partielles par bloc, réduction séquentielle dans l'ordre des blocs.
fn dot(a: &[f64], b: &[f64]) -> f64 {
    let partial: Vec<f64> = a
        .par_chunks(BLOCK)
        .zip(b.par_chunks(BLOCK))
        .map(|(x, y)| x.iter().zip(y).map(|(p, q)| p * q).sum::<f64>())
        .collect();
    partial.iter().sum()
}

fn block_sum(values: &[f64]) -> f64 {
    block_reduce(values, |chunk| chunk.iter().sum())
}

/// Réduction par blocs fixes, identique quel que soit le nombre de threads.
fn block_reduce<F>(values: &[f64], reduce: F) -> f64
where
    F: Fn(&[f64]) -> f64 + Sync,
{
    let partial: Vec<f64> = values.par_chunks(BLOCK).map(&reduce).collect();
    partial.iter().sum()
}

/// Écrit `out` par blocs fixes et renvoie la somme des valeurs retournées par bloc, réduites dans l'ordre.
fn block_map_mut<F>(out: &mut [f64], f: F) -> f64
where
    F: Fn(usize, &mut [f64]) -> f64 + Sync,
{
    let partial: Vec<f64> = out
        .par_chunks_mut(BLOCK)
        .enumerate()
        .map(|(b, chunk)| f(b * BLOCK, chunk))
        .collect();
    partial.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::grid::Grid;
    use crate::shared::units::R_EARTH_M;

    fn mesh(level: u8) -> Mesh {
        Mesh::new(&Grid::new(level, R_EARTH_M)).expect("maillage géodésique")
    }

    fn varied_rhs(n: usize) -> Vec<f64> {
        (0..n)
            .map(|i| (((i * 2654435761) % 997) as f64 / 997.0) - 0.5)
            .collect()
    }

    fn pool(threads: usize) -> rayon::ThreadPool {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("pool rayon")
    }

    #[test]
    fn converges_to_tight_tolerance_with_enough_iterations() {
        let mesh = mesh(3);
        let mass = vec![1.0; mesh.cell_count()];
        let op = ImplicitOperator::new(&mesh, &mass, 0.5);
        let rhs = varied_rhs(mesh.cell_count());
        let mut x = vec![0.0; rhs.len()];
        let report = solve_with(&op, &rhs, &mut x, 2000, 1e-12);
        assert!(report.relative_residual < 1e-12 && !report.cap_reached, "{report:?}");
    }

    #[test]
    fn cap_is_reported_when_iterations_are_insufficient() {
        let mesh = mesh(5);
        let mass = vec![0.5; mesh.cell_count()];
        let op = ImplicitOperator::new(&mesh, &mass, 0.555);
        let rhs = varied_rhs(mesh.cell_count());
        let mut x = vec![0.0; rhs.len()];
        let report = solve_with(&op, &rhs, &mut x, 2, 1e-12);
        assert_eq!(report.iterations, 2);
        assert!(report.cap_reached, "{report:?}");
    }

    #[test]
    fn default_solve_respects_iteration_cap() {
        let mesh = mesh(5);
        let mass = vec![0.5; mesh.cell_count()];
        let op = ImplicitOperator::new(&mesh, &mass, 0.555);
        let rhs = varied_rhs(mesh.cell_count());
        let mut x = vec![0.0; rhs.len()];
        let report = solve(&op, &rhs, &mut x);
        assert!(report.iterations <= CG_MAX_ITER, "{report:?}");
    }

    #[test]
    fn energy_is_conserved_exactly_after_solve() {
        let mesh = mesh(5);
        let mass: Vec<f64> = (0..mesh.cell_count()).map(|i| 1.0 + (i % 7) as f64).collect();
        let op = ImplicitOperator::new(&mesh, &mass, 0.555);
        let rhs = varied_rhs(mesh.cell_count());
        let mut x = vec![0.0; rhs.len()];
        solve(&op, &rhs, &mut x);
        let mut ax = vec![0.0; x.len()];
        op.apply(&x, &mut ax);
        let residual_sum: f64 = rhs.iter().zip(&ax).map(|(b, a)| b - a).sum();
        let scale: f64 = rhs.iter().map(|b| b.abs()).sum();
        assert!(residual_sum.abs() <= 1e-12 * scale, "écart d'énergie {residual_sum}");
    }

    #[test]
    fn warm_start_needs_no_more_iterations_than_cold_start() {
        let mesh = mesh(5);
        let mass = vec![2.0; mesh.cell_count()];
        let op = ImplicitOperator::new(&mesh, &mass, 0.555);
        let rhs = varied_rhs(mesh.cell_count());
        let mut cold = vec![0.0; rhs.len()];
        let cold_report = solve(&op, &rhs, &mut cold);
        let mut warm = cold.clone();
        let warm_report = solve(&op, &rhs, &mut warm);
        assert!(
            warm_report.iterations <= cold_report.iterations,
            "{cold_report:?} {warm_report:?}"
        );
    }

    #[test]
    fn zero_diffusion_reduces_to_mass_division() {
        let mesh = mesh(3);
        let mass = vec![3.0; mesh.cell_count()];
        let op = ImplicitOperator::new(&mesh, &mass, 0.0);
        let rhs = mesh.area_sr().to_vec();
        let mut x = vec![0.7; rhs.len()];
        solve(&op, &rhs, &mut x);
        assert!(x.iter().all(|v| (v - 1.0 / 3.0).abs() < 1e-9));
    }

    #[test]
    fn result_is_bitwise_identical_for_one_and_six_threads() {
        let mesh = mesh(5);
        let mass: Vec<f64> = (0..mesh.cell_count()).map(|i| 0.8 + (i % 5) as f64).collect();
        let op = ImplicitOperator::new(&mesh, &mass, 0.555);
        let rhs = varied_rhs(mesh.cell_count());
        let run = |threads: usize| {
            pool(threads).install(|| {
                let mut x = vec![0.0; rhs.len()];
                let report = solve(&op, &rhs, &mut x);
                (x, report)
            })
        };
        let (single, single_report) = run(1);
        let (parallel, parallel_report) = run(6);
        assert_eq!(single_report, parallel_report);
        assert!(single.iter().zip(&parallel).all(|(a, b)| a.to_bits() == b.to_bits()));
    }
}
