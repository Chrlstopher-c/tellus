//! Loi de Damuth (1981) : densité maximale d'une espèce ∝ M^(−3/4). Sert de plafond territorial par cohorte.

use super::metabolism::quarter_root;

/// Densité maximale d'individus pour M = 1 kgC, par km² (constante documentée ; ordre de grandeur des mammifères).
pub const DAMUTH_D0_PER_KM2: f64 = 1.0;
/// Pente de Damuth (log-log de la densité en fonction de la masse), appliquée via la racine quatrième.
pub const DAMUTH_SLOPE: f64 = -0.75;
const M2_PER_KM2: f64 = 1e6;

/// Densité maximale d'individus par km² pour une masse individuelle `mass_kgc` : D0 · M^(−3/4).
pub fn max_density_per_km2(mass_kgc: f64) -> f64 {
    let q = quarter_root(mass_kgc);
    DAMUTH_D0_PER_KM2 / (q * q * q)
}

/// Effectif maximal d'une cohorte dans une cellule de `area_m2` : plafond de territorialité.
pub fn territorial_cap(mass_kgc: f64, area_m2: f64) -> f64 {
    max_density_per_km2(mass_kgc) * area_m2 / M2_PER_KM2
}
