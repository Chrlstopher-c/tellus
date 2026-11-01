//! Métabolisme (théorie métabolique : B = B0 · M^(3/4) · e^(−E/kT)), mortalité et consommation maximale.
//! Les exposants ±1/4 et 3/4 passent par la racine quatrième (pas de `powf` dans la boucle chaude).

use crate::shared::units::BOLTZMANN_EV;

/// Énergie d'activation moyenne des processus respiratoires, eV (vivant.md § 2.2).
pub const ACTIVATION_EV: f64 = 0.65;
const T_REF_K: f64 = 288.15;
/// Respiration par individu à 15 °C pour 1 kgC, kgC par an.
const METABOLIC_B0: f64 = 1.0;
/// Mortalité de fond par individu à 15 °C pour 1 kgC, par an.
const MORTALITY_B0: f64 = 0.1;
/// Consommation maximale rapportée à la respiration (ratio de Holling, sans saturation).
const INTAKE_FACTOR: f64 = 6.0;

/// Facteur thermique relatif à 15 °C : e^(−E/kT) normalisé.
pub fn arrhenius(temp_c: f64) -> f64 {
    let t_k = temp_c + 273.15;
    (ACTIVATION_EV / BOLTZMANN_EV * (1.0 / T_REF_K - 1.0 / t_k)).exp()
}

/// Racine quatrième de la masse : M^(1/4), base de tous les exposants de la MTE.
pub fn quarter_root(mass_kgc: f64) -> f64 {
    mass_kgc.max(1e-300).sqrt().sqrt()
}

/// Respiration par individu (kgC/an) à facteur thermique `arr` donné.
pub fn metabolic_rate_at(mass_kgc: f64, arr: f64) -> f64 {
    let q = quarter_root(mass_kgc);
    METABOLIC_B0 * q * q * q * arr
}

/// Mortalité de fond (1/an) à facteur thermique `arr` donné, ∝ M^(−1/4).
pub fn mortality_at(mass_kgc: f64, arr: f64) -> f64 {
    MORTALITY_B0 / quarter_root(mass_kgc) * arr
}

/// Consommation maximale par individu (kgC/an) à facteur thermique `arr` donné.
pub fn max_intake_at(mass_kgc: f64, arr: f64) -> f64 {
    INTAKE_FACTOR * metabolic_rate_at(mass_kgc, arr)
}

/// Respiration d'un individu de masse `mass_kgc` à la température `temp_c`, kgC par an.
pub fn metabolic_rate(mass_kgc: f64, temp_c: f64) -> f64 {
    metabolic_rate_at(mass_kgc, arrhenius(temp_c))
}

/// Mortalité de fond par an à la température `temp_c`.
pub fn mortality_rate(mass_kgc: f64, temp_c: f64) -> f64 {
    mortality_at(mass_kgc, arrhenius(temp_c))
}

/// Consommation maximale par individu à la température `temp_c`, kgC par an.
pub fn max_intake_rate(mass_kgc: f64, temp_c: f64) -> f64 {
    max_intake_at(mass_kgc, arrhenius(temp_c))
}
