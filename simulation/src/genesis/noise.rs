//! Bruit de valeur 3D évalué sur la sphère unitaire (pas de couture, pas de pôle) : fBm et crêtes.

use super::vec3::{scale, Vec3};
use crate::shared::rng::{hash, unit_f64};

const OCTAVE_TICK: u64 = 0x4E4F_4953;

fn lattice(seed: u64, ix: i64, iy: i64, iz: i64) -> f64 {
    2.0 * unit_f64(seed, ix as u64, iy as u32, iz as u32) - 1.0
}

fn fade(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Bruit de valeur dans [-1, 1], interpolé trilinéairement avec une courbe de Hermite.
pub(super) fn value_noise(seed: u64, p: Vec3) -> f64 {
    let base = p.map(f64::floor);
    let (ix, iy, iz) = (base[0] as i64, base[1] as i64, base[2] as i64);
    let (fx, fy, fz) = (fade(p[0] - base[0]), fade(p[1] - base[1]), fade(p[2] - base[2]));
    let corner = |dx: i64, dy: i64, dz: i64| lattice(seed, ix + dx, iy + dy, iz + dz);
    let x00 = lerp(corner(0, 0, 0), corner(1, 0, 0), fx);
    let x10 = lerp(corner(0, 1, 0), corner(1, 1, 0), fx);
    let x01 = lerp(corner(0, 0, 1), corner(1, 0, 1), fx);
    let x11 = lerp(corner(0, 1, 1), corner(1, 1, 1), fx);
    lerp(lerp(x00, x10, fy), lerp(x01, x11, fy), fz)
}

/// Graine propre à l'octave `k` d'un flux de bruit.
fn octave_seed(seed: u64, k: u32) -> u64 {
    hash(seed, OCTAVE_TICK, k, 0)
}

/// Somme d'octaves (fréquence doublée, amplitude divisée par deux), `shape` appliqué à chaque bruit de valeur,
/// normalisée par la somme des amplitudes.
fn octave_sum(seed: u64, p: Vec3, base_freq: f64, octaves: u32, shape: impl Fn(f64) -> f64) -> f64 {
    let (mut sum, mut amplitude, mut total, mut freq) = (0.0, 1.0, 0.0, base_freq);
    for k in 0..octaves {
        sum += amplitude * shape(value_noise(octave_seed(seed, k), scale(p, freq)));
        total += amplitude;
        amplitude *= 0.5;
        freq *= 2.0;
    }
    sum / total
}

/// Mouvement fractionnaire dans [-1, 1].
pub(super) fn fbm(seed: u64, p: Vec3, base_freq: f64, octaves: u32) -> f64 {
    octave_sum(seed, p, base_freq, octaves, |n| n)
}

/// Crêtes (bruit replié, maxima marqués) dans [0, 1].
pub(super) fn ridged(seed: u64, p: Vec3, base_freq: f64, octaves: u32) -> f64 {
    octave_sum(seed, p, base_freq, octaves, |n| (1.0 - n.abs()).powi(2))
}
