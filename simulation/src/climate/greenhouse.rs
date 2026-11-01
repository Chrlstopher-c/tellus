//! Effet de serre. Terre : `OLR = A + B·T − ΔF_CO₂` (T en °C, ΔF = 5,35·ln(CO₂/280)).
//! Autres corps : atmosphère grise, `OLR = σT⁴ / (1 + 3τ/4)`.
//! τ d'un corps réel : calibré sur sa température observée (`calibrate_tau`), puis mis à l'échelle du CO₂ par
//! `scaled_tau`. τ d'un monde généré : `tau_for_world(P, x_CO₂)` = f(P)·g(x_CO₂), loi à trois paramètres ajustée sur
//! les ancres Terre, Mars et Vénus (voir la documentation de `tau_for_world`).

use super::body::Body;
use crate::shared::units::SIGMA;
use std::fmt;

/// Concentration de référence du forçage terrestre, ppm.
pub const CO2_REF_PPM: f64 = 280.0;
/// Coefficient du forçage logarithmique du CO₂, W m⁻².
pub const CO2_FORCING_COEF_W_M2: f64 = 5.35;
/// Ordonnée à l'origine de l'émission terrestre, W m⁻² (climlab, calée sur 288 K).
pub const EARTH_OLR_A_W_M2: f64 = 210.0;
/// Pente de l'émission terrestre, W m⁻² K⁻¹.
pub const EARTH_OLR_B_W_M2_K: f64 = 2.0;
/// Coefficient k de la loi τ = k·P^a·x^b (P en bar, x fraction molaire de CO₂).
pub const TAU_LAW_SCALE: f64 = 4.966_040;
/// Exposant en pression de la loi τ.
pub const TAU_PRESSURE_EXPONENT: f64 = 0.752_234;
/// Exposant en CO₂ de la loi τ, aussi utilisé pour la mise à l'échelle d'un corps réel.
pub const TAU_CO2_EXPONENT: f64 = 0.231_124;
/// Borne haute de la recherche de τ (Vénus ≈ 148).
const TAU_SEARCH_MAX: f64 = 10_000.0;
/// Itérations de bisection : 10⁴ / 2¹⁰⁰ est sous le plancher flottant utile.
const BISECTION_STEPS: u32 = 100;

#[derive(Debug, Clone, PartialEq)]
pub enum GreenhouseError {
    NoSurfaceTarget(String),
}

impl fmt::Display for GreenhouseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSurfaceTarget(id) => write!(
                f,
                "corps '{id}' : température de surface absente, calibration impossible"
            ),
        }
    }
}

impl std::error::Error for GreenhouseError {}

/// Forçage du CO₂ par rapport à la référence, W m⁻².
pub fn co2_forcing(co2_ppm: f64) -> f64 {
    CO2_FORCING_COEF_W_M2 * (co2_ppm / CO2_REF_PPM).ln()
}

/// Émission de l'atmosphère grise au sommet, W m⁻², pour une surface à `t_k` kelvins.
pub fn grey_olr(t_k: f64, tau: f64) -> f64 {
    SIGMA * t_k.powi(4) / (1.0 + 0.75 * tau)
}

/// Dérivée de l'émission grise par rapport à la température, W m⁻² K⁻¹.
pub fn grey_olr_slope(t_k: f64, tau: f64) -> f64 {
    4.0 * SIGMA * t_k.powi(3) / (1.0 + 0.75 * tau)
}

/// Température de surface à l'équilibre pour un flux absorbé moyen `absorbed` (W m⁻²).
pub fn grey_surface_temp_k(absorbed: f64, tau: f64) -> f64 {
    (absorbed * (1.0 + 0.75 * tau) / SIGMA).powf(0.25)
}

/// τ tel que la température de surface globale d'équilibre égale la température observée du corps.
/// Bisection sur le bilan global 0-D (flux absorbé moyen S(1−A)/4) : le gradient latitudinal est ignoré ici
/// et mesuré à part par les tests de l'EBM complet.
pub fn calibrate_tau(body: &Body) -> Result<f64, GreenhouseError> {
    let target = body
        .surface_temp_mean_k
        .ok_or_else(|| GreenhouseError::NoSurfaceTarget(body.id.clone()))?;
    let absorbed = body.solar_flux_wm2 * (1.0 - body.bond_albedo) / 4.0;
    let (mut low, mut high) = (0.0_f64, TAU_SEARCH_MAX);
    for _ in 0..BISECTION_STEPS {
        let mid = 0.5 * (low + high);
        if grey_surface_temp_k(absorbed, mid) < target {
            low = mid;
        } else {
            high = mid;
        }
    }
    Ok(0.5 * (low + high))
}

/// Mise à l'échelle de τ d'un corps réel quand la concentration de CO₂ change : `τ_ref·(CO₂/CO₂_ref)^b`.
/// Une concentration nulle donne τ = 0 ; un corps sans CO₂ de référence (Mercure, Lune) garde `tau_ref`.
pub fn scaled_tau(tau_ref: f64, co2_ppm: f64, co2_ref_ppm: f64) -> f64 {
    if co2_ppm <= 0.0 {
        return 0.0;
    }
    if co2_ref_ppm <= 0.0 {
        return tau_ref;
    }
    tau_ref * (co2_ppm / co2_ref_ppm).powf(TAU_CO2_EXPONENT)
}

/// Profondeur optique grise d'un monde généré : `τ = f(P) · g(x)`, `f(P) = k·P^a`, `g(x) = x^b`.
///
/// Ancres (τ calibrés par `calibrate_tau`, puis `tau_for_world` reproduit ces valeurs) :
/// - Terre : P = 1,014 bar, x = 4,2·10⁻⁴, τ ≈ 0,832 ;
/// - Mars : P = 0,00636 bar, x = 0,951, τ ≈ 0,109 ;
/// - Vénus : P = 92 bar, x = 0,965, τ ≈ 147,8.
///
/// Trois paramètres (k, a, b) pour trois ancres : l'ajustement est exact, il ne valide rien au-delà. La loi
/// est donc une interpolation, pas une validation : un monde hors de l'espace (P, x) couvert par ces trois
/// corps est une extrapolation à signaler.
pub fn tau_for_world(pressure_bar: f64, co2_fraction: f64) -> f64 {
    if pressure_bar <= 0.0 || co2_fraction <= 0.0 {
        return 0.0;
    }
    TAU_LAW_SCALE * pressure_bar.powf(TAU_PRESSURE_EXPONENT) * co2_fraction.powf(TAU_CO2_EXPONENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::climate::body::body;

    fn calibrated(id: &str) -> (Body, f64) {
        let b = body(id).expect("corps réel");
        let tau = calibrate_tau(&b).expect("cible de surface");
        (b, tau)
    }

    #[test]
    fn v08_doubling_forcing_is_3_71() {
        assert!((co2_forcing(560.0) - 3.71).abs() <= 0.01, "{}", co2_forcing(560.0));
        assert!(co2_forcing(280.0).abs() < 1e-12);
    }

    #[test]
    fn v11_v12_earth_equilibrium_and_tau() {
        let (b, tau) = calibrated("earth");
        assert!((b.equilibrium_temperature_k() - 255.6).abs() <= 1.0);
        assert!((tau - 0.8).abs() <= 0.2, "τ Terre {tau}");
    }

    #[test]
    fn v11_v12_mars_equilibrium_and_tau() {
        let (b, tau) = calibrated("mars");
        assert!((b.equilibrium_temperature_k() - 210.0).abs() <= 3.0);
        assert!((tau - 0.1).abs() <= 0.05, "τ Mars {tau}");
    }

    #[test]
    fn v11_v12_venus_equilibrium_and_tau() {
        let (b, tau) = calibrated("venus");
        assert!((b.equilibrium_temperature_k() - 226.0).abs() <= 3.0);
        assert!((tau - 150.0).abs() <= 20.0, "τ Vénus {tau}");
    }

    #[test]
    fn world_law_reproduces_the_three_anchors_exactly() {
        for id in ["earth", "mars", "venus"] {
            let (b, tau) = calibrated(id);
            let pressure = b.surface_pressure_bar.expect("pression de surface");
            let fraction = b.co2_fraction.expect("fraction de CO₂");
            let law = tau_for_world(pressure, fraction);
            assert!((law - tau).abs() <= 1e-4 * tau, "{id} : loi {law}, calibré {tau}");
        }
    }

    #[test]
    fn world_law_is_separable_in_pressure_and_co2() {
        let (p, x) = (0.5, 0.01);
        let ratio_p = tau_for_world(2.0 * p, x) / tau_for_world(p, x);
        let ratio_x = tau_for_world(p, 2.0 * x) / tau_for_world(p, x);
        assert!((ratio_p - 2.0_f64.powf(TAU_PRESSURE_EXPONENT)).abs() < 1e-12);
        assert!((ratio_x - 2.0_f64.powf(TAU_CO2_EXPONENT)).abs() < 1e-12);
    }

    #[test]
    fn scaled_tau_grows_with_co2_and_vanishes_without_it() {
        let base = scaled_tau(0.8, 280.0, 280.0);
        assert!((base - 0.8).abs() < 1e-12);
        assert!(scaled_tau(0.8, 560.0, 280.0) > base);
        assert_eq!(scaled_tau(0.8, 0.0, 280.0), 0.0);
    }

    #[test]
    fn grey_balance_closes() {
        let absorbed = 240.0;
        let tau = 0.8;
        let t = grey_surface_temp_k(absorbed, tau);
        assert!((grey_olr(t, tau) - absorbed).abs() < 1e-9);
    }

    #[test]
    fn grey_slope_matches_finite_difference() {
        let (t, tau) = (288.0, 0.8);
        let numeric = (grey_olr(t + 1e-4, tau) - grey_olr(t - 1e-4, tau)) / 2e-4;
        assert!((numeric - grey_olr_slope(t, tau)).abs() < 1e-6);
    }
}
