//! Génome : 18 traits normalisés dans [0, 1], modèle des moments (docs/recherche/vivant.md § 5).

use crate::shared::rng::unit_f64;

/// Nombre de traits par lignée.
pub const NT: usize = 18;
/// Traits d'une lignée, normalisés dans [0, 1].
pub type Genome = [f64; NT];

/// Index des traits (vivant.md § 5.2, numérotation à partir de 0).
pub mod idx {
    pub const LOG_MASS: usize = 0;
    pub const BIRTH_RATIO: usize = 1;
    pub const DIET: usize = 2;
    pub const T_OPT: usize = 3;
    pub const T_WIDTH: usize = 4;
    pub const WATER: usize = 5;
    pub const MOBILITY: usize = 6;
    pub const REPRO: usize = 7;
    pub const LONGEVITY: usize = 8;
    pub const RECOMB: usize = 9;
    pub const MEDIUM: usize = 10;
    pub const DEFENSE: usize = 11;
    pub const HUNT: usize = 12;
    pub const GREGARIOUS: usize = 13;
    pub const FIRE: usize = 14;
    pub const GROWTH: usize = 15;
    pub const PHOTO: usize = 16;
    pub const ADHESION: usize = 17;
}

/// Traits de compatibilité, base de la distance génétique (spéciation).
pub const COMPAT_TRAITS: [usize; 3] = [idx::RECOMB, idx::GREGARIOUS, idx::DIET];
/// Écart-type de mutation par trait et par naissance : σ_m · √μ_mut (σ_m = 0,1 ; μ_mut = 0,01).
pub const MUTATION_SD: f64 = 0.01;
const LOG_MASS_MIN: f64 = -6.0;
const LOG_MASS_MAX: f64 = 2.0;

/// Réflexion dans [0, 1] (périodique de période 2), jamais de troncature.
pub fn reflect(x: f64) -> f64 {
    let m = x.rem_euclid(2.0);
    if m > 1.0 {
        2.0 - m
    } else {
        m
    }
}

/// Masse adulte en kgC : de 1 µg à 100 kg de carbone, échelle logarithmique du trait 0.
pub fn adult_mass_kgc(genome: &Genome) -> f64 {
    10f64.powf(LOG_MASS_MIN + genome[idx::LOG_MASS] * (LOG_MASS_MAX - LOG_MASS_MIN))
}

/// Fraction de la masse adulte à la naissance (1 % à 20 %).
pub fn birth_fraction(genome: &Genome) -> f64 {
    0.01 + 0.19 * genome[idx::BIRTH_RATIO]
}

/// Fraction de l'excédent d'énergie allouée à la reproduction (5 % à 55 %).
pub fn repro_fraction(genome: &Genome) -> f64 {
    0.05 + 0.5 * genome[idx::REPRO]
}

/// Taux de dispersion annuel (2 % à 32 % de l'abondance d'une cellule).
pub fn mobility_rate(genome: &Genome) -> f64 {
    0.02 + 0.3 * genome[idx::MOBILITY]
}

/// Longévité de référence en années (1 à 30 ans).
pub fn longevity_yr(genome: &Genome) -> f64 {
    1.0 + 29.0 * genome[idx::LONGEVITY]
}

/// Distance génétique RMS sur les traits de compatibilité, dans [0, 1].
pub fn genetic_distance(a: &Genome, b: &Genome) -> f64 {
    let sum: f64 = COMPAT_TRAITS.iter().map(|&i| (a[i] - b[i]).powi(2)).sum();
    (sum / COMPAT_TRAITS.len() as f64).sqrt()
}

/// Gaussienne standard déterministe (Box-Muller) à partir de deux tirages du compteur.
pub fn gaussian(seed: u64, tick: u64, cell: u32, stream: u32) -> f64 {
    let u1 = unit_f64(seed, tick, cell, stream).max(1e-300);
    let u2 = unit_f64(seed, tick, cell, stream.wrapping_add(0x8000_0000));
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

/// Copie mutée du génome parent : perturbation gaussienne par trait, bornes par réflexion.
pub fn mutate(parent: &Genome, seed: u64, tick: u64, cell: u32, stream: u32) -> Genome {
    let mut child = *parent;
    for (i, value) in child.iter_mut().enumerate() {
        let draw = stream.wrapping_mul(64).wrapping_add(i as u32);
        *value = reflect(*value + MUTATION_SD * gaussian(seed, tick, cell, draw));
    }
    child
}
