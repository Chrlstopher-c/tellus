//! Transport de vapeur à l'état stationnaire (climat.md §6). Flux massiques aux arêtes duales en amont (upwind),
//! conservatifs par construction : `G_ji = −G_ij` exactement, donc la vapeur totale ne change que par E et P.
//! Bilan d'une saison, avec σ_i = [W_i > W_sat,i] (condensation au-delà de la saturation) :
//!   `Σ_k G_k(upwind) + A_i (s_i + λ_i + σ_i/τ) W_i = A_i (e_i + σ_i W_sat,i / τ)`.
//! Résolu par BiCGSTAB préconditionné Jacobi, puis itération de Picard sur σ.

use crate::climate::mesh::Mesh;
use rayon::prelude::*;

/// Itérations maximales de BiCGSTAB par résolution et tolérance relative sur le résidu.
const KRYLOV_MAX_ITER: u32 = 400;
const KRYLOV_TOLERANCE: f64 = 1e-8;
/// Passes maximales de Picard sur l'ensemble des cellules saturées.
const PICARD_MAX: u32 = 8;

/// Bilan d'une résolution : itérations cumulées et convergence de l'ensemble saturé.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct SolveStats {
    pub(super) iterations: u32,
    pub(super) converged: bool,
}

/// Flux massique orienté par arête duale : `G_k = R·(ℓ_k n_k)·(V_i + V_j)/2`, en kg s⁻¹ par kg m⁻² de vapeur.
pub(super) fn directed_flux(mesh: &Mesh, winds: &[[f64; 3]]) -> Vec<f64> {
    let offsets = mesh.edge_offsets();
    let neighbours = mesh.neighbor_indices();
    let flows = mesh.edge_flow();
    let radius = mesh.radius_m();
    let mut flux = vec![0.0; neighbours.len()];
    for cell in 0..mesh.cell_count() {
        for k in offsets[cell] as usize..offsets[cell + 1] as usize {
            let other = neighbours[k] as usize;
            let mid: [f64; 3] = std::array::from_fn(|a| 0.5 * (winds[cell][a] + winds[other][a]));
            flux[k] = radius * dot(&flows[k], &mid);
        }
    }
    flux
}

/// Opérateur stationnaire d'une saison : données par cellule et flux sortant par arête.
pub(super) struct Steady<'a> {
    mesh: &'a Mesh,
    /// Partie négative des flux (`min(G, 0)`) : apport venant du voisin, par arête orientée.
    inflow: Vec<f64>,
    /// Diagonale hors condensation : `Σ_k max(G_k, 0) + A_i (s_i + λ_i)`.
    base: Vec<f64>,
    /// Second membre hors condensation : `A_i e_i`, kg s⁻¹.
    source: Vec<f64>,
    /// Aire physique des cellules, m².
    area: Vec<f64>,
    /// Saturation de la colonne, kg m⁻².
    wsat: Vec<f64>,
    /// Temps de conversion de la vapeur en pluie, s.
    rain_time_s: f64,
}

impl<'a> Steady<'a> {
    /// `lin_i = A_i (s_i + λ_i)` et `source_i = A_i e_i` sont fournis par l'appelant.
    pub(super) fn new(
        mesh: &'a Mesh,
        flux: &[f64],
        lin: Vec<f64>,
        source: Vec<f64>,
        wsat: Vec<f64>,
        rain_s: f64,
    ) -> Self {
        let offsets = mesh.edge_offsets();
        let radius = mesh.radius_m();
        let base = (0..mesh.cell_count())
            .map(|cell| {
                let outflow: f64 = (offsets[cell] as usize..offsets[cell + 1] as usize)
                    .map(|k| flux[k].max(0.0))
                    .sum();
                outflow + lin[cell]
            })
            .collect();
        Self {
            mesh,
            inflow: flux.iter().map(|g| g.min(0.0)).collect(),
            base,
            source,
            area: mesh.area_sr().iter().map(|a| a * radius * radius).collect(),
            wsat,
            rain_time_s: rain_s,
        }
    }

    /// Résout le bilan depuis `x` (départ à chaud), met à jour `x` et renvoie les statistiques.
    pub(super) fn solve(&self, x: &mut [f64]) -> SolveStats {
        let mut sat = self.saturated(x);
        let mut stats = SolveStats::default();
        for _ in 0..PICARD_MAX {
            let diag = self.diagonal(&sat);
            let rhs = self.rhs(&sat);
            let (iterations, _) = self.bicgstab(&diag, &rhs, x);
            stats.iterations += iterations;
            let next = self.saturated(x);
            if next == sat {
                stats.converged = self.residual_ok(&diag, &rhs, x);
                return stats;
            }
            sat = next;
        }
        stats
    }

    /// Ensemble saturé pour un champ `x` : `W > W_sat`.
    pub(super) fn saturated(&self, x: &[f64]) -> Vec<bool> {
        x.iter().zip(&self.wsat).map(|(w, s)| w > s).collect()
    }

    fn diagonal(&self, sat: &[bool]) -> Vec<f64> {
        let sigma = 1.0 / self.rain_time_s;
        self.base
            .iter()
            .zip(&self.area)
            .zip(sat)
            .map(|((b, a), saturated)| if *saturated { b + a * sigma } else { *b })
            .collect()
    }

    fn rhs(&self, sat: &[bool]) -> Vec<f64> {
        self.source
            .iter()
            .zip(&self.area)
            .zip(&self.wsat)
            .zip(sat)
            .map(|(((src, a), w), saturated)| if *saturated { src + a * w / self.rain_time_s } else { *src })
            .collect()
    }

    /// `out = D·x + Σ_k min(G_k,0)·x_j` : opérateur parallèle, chaque cellule est indépendante (déterministe).
    fn apply(&self, diag: &[f64], x: &[f64], out: &mut [f64]) {
        let offsets = self.mesh.edge_offsets();
        let neighbours = self.mesh.neighbor_indices();
        out.par_iter_mut().enumerate().for_each(|(i, slot)| {
            let mut acc = diag[i] * x[i];
            for k in offsets[i] as usize..offsets[i + 1] as usize {
                acc += self.inflow[k] * x[neighbours[k] as usize];
            }
            *slot = acc;
        });
    }

    fn residual_ok(&self, diag: &[f64], rhs: &[f64], x: &[f64]) -> bool {
        let mut ax = vec![0.0; x.len()];
        self.apply(diag, x, &mut ax);
        let residual: Vec<f64> = rhs.iter().zip(&ax).map(|(b, a)| b - a).collect();
        norm(&residual) <= KRYLOV_TOLERANCE * norm(rhs).max(f64::MIN_POSITIVE)
    }

    fn bicgstab(&self, diag: &[f64], rhs: &[f64], x: &mut [f64]) -> (u32, bool) {
        let target = KRYLOV_TOLERANCE * norm(rhs).max(f64::MIN_POSITIVE);
        let mut work = Krylov::start(self, diag, rhs, x);
        for iteration in 0..KRYLOV_MAX_ITER {
            if norm(&work.r) <= target {
                return (iteration, true);
            }
            if !work.step(self, diag, x) {
                return (iteration, false);
            }
        }
        (KRYLOV_MAX_ITER, norm(&work.r) <= target)
    }
}

/// Vecteurs de travail de BiCGSTAB et scalaires de récurrence.
struct Krylov {
    r: Vec<f64>,
    r_hat: Vec<f64>,
    p: Vec<f64>,
    v: Vec<f64>,
    rho: f64,
    alpha: f64,
    omega: f64,
}

impl Krylov {
    fn start(op: &Steady<'_>, diag: &[f64], rhs: &[f64], x: &[f64]) -> Self {
        let mut ax = vec![0.0; x.len()];
        op.apply(diag, x, &mut ax);
        let r: Vec<f64> = rhs.iter().zip(&ax).map(|(b, a)| b - a).collect();
        Self {
            r_hat: r.clone(),
            r,
            p: vec![0.0; x.len()],
            v: vec![0.0; x.len()],
            rho: 1.0,
            alpha: 1.0,
            omega: 1.0,
        }
    }

    /// Un pas de BiCGSTAB ; `false` si la récurrence casse (division nulle ou valeurs non finies).
    fn step(&mut self, op: &Steady<'_>, diag: &[f64], x: &mut [f64]) -> bool {
        let rho_new = dot(&self.r_hat, &self.r);
        if !rho_new.is_finite() || rho_new == 0.0 {
            return false;
        }
        let beta = (rho_new / self.rho) * (self.alpha / self.omega);
        let omega = self.omega;
        for ((p, r), v) in self.p.iter_mut().zip(&self.r).zip(&self.v) {
            *p = r + beta * (*p - omega * v);
        }
        let y = jacobi(diag, &self.p);
        op.apply(diag, &y, &mut self.v);
        let denominator = dot(&self.r_hat, &self.v);
        if !denominator.is_finite() || denominator == 0.0 {
            return false;
        }
        self.alpha = rho_new / denominator;
        let s: Vec<f64> = self.r.iter().zip(&self.v).map(|(r, v)| r - self.alpha * v).collect();
        self.rho = rho_new;
        self.stabilise(op, diag, x, &y, &s)
    }

    /// Seconde moitié du pas : correction stabilisante ω et mise à jour de `x` et du résidu.
    fn stabilise(&mut self, op: &Steady<'_>, diag: &[f64], x: &mut [f64], y: &[f64], s: &[f64]) -> bool {
        let z = jacobi(diag, s);
        let mut t = vec![0.0; s.len()];
        op.apply(diag, &z, &mut t);
        let tt = dot(&t, &t);
        if tt <= f64::MIN_POSITIVE {
            add_scaled(x, y, self.alpha);
            self.r.copy_from_slice(s);
            return true;
        }
        self.omega = dot(&t, s) / tt;
        add_scaled(x, y, self.alpha);
        add_scaled(x, &z, self.omega);
        self.r = s.iter().zip(&t).map(|(s, t)| s - self.omega * t).collect();
        self.omega.is_finite() && self.omega != 0.0
    }
}

/// Préconditionnement de Jacobi : `z_i = r_i / D_i` (identité si la diagonale n'est pas strictement positive).
fn jacobi(diag: &[f64], r: &[f64]) -> Vec<f64> {
    r.iter()
        .zip(diag)
        .map(|(v, d)| if *d > 0.0 { v / d } else { *v })
        .collect()
}

fn add_scaled(target: &mut [f64], direction: &[f64], scale: f64) {
    for (t, d) in target.iter_mut().zip(direction) {
        *t += scale * d;
    }
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(p, q)| p * q).sum()
}

fn norm(a: &[f64]) -> f64 {
    dot(a, a).sqrt()
}
