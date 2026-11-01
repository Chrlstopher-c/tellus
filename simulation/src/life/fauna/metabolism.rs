//! Métabolisme (théorie métabolique : B = B0 · M^(3/4) · e^(−E/kT)), mortalité et consommation maximale.

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

/// Respiration d'un individu de masse `mass_kgc`, kgC par an.
pub fn metabolic_rate(mass_kgc: f64, temp_c: f64) -> f64 {
    METABOLIC_B0 * mass_kgc.powf(0.75) * arrhenius(temp_c)
}

/// Mortalité de fond par an, proportionnelle à M^(−1/4) (temps de génération de la MTE).
pub fn mortality_rate(mass_kgc: f64, temp_c: f64) -> f64 {
    MORTALITY_B0 * mass_kgc.max(1e-300).powf(-0.25) * arrhenius(temp_c)
}

/// Consommation maximale par individu, kgC par an, avant saturation de Holling.
pub fn max_intake_rate(mass_kgc: f64, temp_c: f64) -> f64 {
    INTAKE_FACTOR * metabolic_rate(mass_kgc, temp_c)
}
