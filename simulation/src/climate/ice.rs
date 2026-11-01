//! Glace et albédo : rampe linéaire de glace autour de −10 °C sur 10 K, albédo de surface libre de Budyko–Sellers
//! `a0 + a2·P2(sin φ)` (cœur terrestre, climlab). La glace de mer et la glace continentale partagent la même fraction.

/// Seuil de gel, °C (centre de la rampe).
pub const ICE_THRESHOLD_C: f64 = -10.0;
/// Largeur de la rampe de glace, K : glace pleine en dessous de T_f − 5 K, nulle au-dessus de T_f + 5 K.
pub const ICE_RAMP_K: f64 = 10.0;
/// Albédo de la glace et de la neige, sans dimension.
pub const ALBEDO_ICE: f64 = 0.62;
/// Ordonnée à l'origine de l'albédo libre zonal (Budyko).
pub const BUDYKO_A0: f64 = 0.30;
/// Coefficient de P2 de l'albédo libre zonal (Budyko).
pub const BUDYKO_A2: f64 = 0.078;

/// Fraction de glace dans [0, 1] pour une température de surface en °C.
pub fn ice_fraction(surface_c: f64) -> f64 {
    ((ICE_THRESHOLD_C + 0.5 * ICE_RAMP_K - surface_c) / ICE_RAMP_K).clamp(0.0, 1.0)
}

/// Albédo libre (sans glace) d'une latitude, `a0 + a2·P2(sin φ)`.
pub fn budyko_free_albedo(sin_lat: f64) -> f64 {
    let p2 = 0.5 * (3.0 * sin_lat * sin_lat - 1.0);
    BUDYKO_A0 + BUDYKO_A2 * p2
}

/// Albédo de surface mélangé : `(1 − f)·α_libre + f·α_glace`.
pub fn surface_albedo(ice: f64, free_albedo: f64) -> f64 {
    (1.0 - ice) * free_albedo + ice * ALBEDO_ICE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v15_ice_fraction_bounds() {
        assert_eq!(ice_fraction(ICE_THRESHOLD_C + 5.0 + 1e-9), 0.0);
        assert_eq!(ice_fraction(ICE_THRESHOLD_C + 5.0 + 20.0), 0.0);
        assert_eq!(ice_fraction(ICE_THRESHOLD_C - 5.0 - 1e-9), 1.0);
        assert_eq!(ice_fraction(ICE_THRESHOLD_C - 40.0), 1.0);
    }

    #[test]
    fn ice_fraction_is_monotone_decreasing_in_ramp() {
        let mut previous = 1.0;
        for step in 0..=300 {
            let t = -20.0 + f64::from(step) * 0.1;
            let f = ice_fraction(t);
            assert!(f <= previous + 1e-15 && (0.0..=1.0).contains(&f), "T={t} f={f}");
            previous = f;
        }
    }

    #[test]
    fn free_albedo_global_mean_is_budyko_a0() {
        let samples = 10_000;
        let mean: f64 = (0..samples)
            .map(|k| budyko_free_albedo(-1.0 + 2.0 * (f64::from(k) + 0.5) / f64::from(samples)))
            .sum::<f64>()
            / f64::from(samples);
        assert!((mean - BUDYKO_A0).abs() < 1e-6, "{mean}");
    }

    #[test]
    fn surface_albedo_interpolates_between_free_and_ice() {
        assert!((surface_albedo(0.0, 0.3) - 0.3).abs() < 1e-12);
        assert!((surface_albedo(1.0, 0.3) - ALBEDO_ICE).abs() < 1e-12);
        assert!((surface_albedo(0.5, 0.3) - 0.46).abs() < 1e-12);
    }
}
