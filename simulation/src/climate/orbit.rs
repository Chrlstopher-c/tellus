//! Orbite képlérienne : anomalie moyenne → excentrique → vraie, puis distance et déclinaison solaires.

use std::f64::consts::{PI, TAU};

/// Tolérance de Newton sur l'équation de Kepler, rad.
const KEPLER_TOLERANCE: f64 = 1e-12;
const KEPLER_MAX_ITER: u32 = 50;
/// Jour de l'année (0 = 1er janvier) du passage au périhélie : valeur terrestre (≈ 3 janvier), reprise pour
/// tous les corps faute de source par corps. Elle décale la saison, sans toucher aux moyennes annuelles.
pub const PERIHELION_DAY: f64 = 3.0;

/// Paramètres képlériens d'un corps, angles en radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orbit {
    pub eccentricity: f64,
    pub obliquity_rad: f64,
    pub perihelion_lon_rad: f64,
    pub period_days: f64,
}

/// État solaire vu depuis le corps à un instant donné.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbitState {
    /// (a/r)² : facteur multiplicatif de l'irradiance reçue à distance moyenne a.
    pub distance_factor: f64,
    /// Déclinaison du Soleil, rad.
    pub declination_rad: f64,
    /// Longitude écliptique géocentrique du Soleil depuis l'équinoxe de printemps, rad dans [0, 2π).
    pub solar_longitude_rad: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrbitError {
    NoConvergence { mean_anomaly: f64, eccentricity: f64 },
}

impl std::fmt::Display for OrbitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoConvergence {
                mean_anomaly,
                eccentricity,
            } => {
                write!(
                    f,
                    "équation de Kepler non convergée (M={mean_anomaly}, e={eccentricity})"
                )
            }
        }
    }
}

impl std::error::Error for OrbitError {}

/// Résout E − e·sin E = M par Newton ; M est ramené dans [0, 2π).
pub fn eccentric_anomaly(mean_anomaly: f64, eccentricity: f64) -> Result<f64, OrbitError> {
    let mean = mean_anomaly.rem_euclid(TAU);
    let mut big_e = mean + eccentricity * mean.sin();
    for _ in 0..KEPLER_MAX_ITER {
        let residual = big_e - eccentricity * big_e.sin() - mean;
        let slope = 1.0 - eccentricity * big_e.cos();
        let step = residual / slope;
        big_e -= step;
        if step.abs() < KEPLER_TOLERANCE {
            return Ok(big_e);
        }
    }
    Err(OrbitError::NoConvergence {
        mean_anomaly,
        eccentricity,
    })
}

/// Anomalie vraie ν depuis l'anomalie excentrique E.
pub fn true_anomaly(big_e: f64, eccentricity: f64) -> f64 {
    let x = (1.0 - eccentricity * eccentricity).sqrt() * big_e.sin();
    let y = big_e.cos() - eccentricity;
    x.atan2(y)
}

impl Orbit {
    /// État solaire au jour `day` de l'année (fractionnaire, 0 = 1er janvier).
    pub fn state_at(&self, day: f64) -> Result<OrbitState, OrbitError> {
        let mean = TAU * (day - PERIHELION_DAY) / self.period_days;
        let big_e = eccentric_anomaly(mean, self.eccentricity)?;
        let nu = true_anomaly(big_e, self.eccentricity);
        let longitude = (nu + self.perihelion_lon_rad + PI).rem_euclid(TAU);
        let declination = (self.obliquity_rad.sin() * longitude.sin()).asin();
        let ratio = (1.0 + self.eccentricity * nu.cos()) / (1.0 - self.eccentricity * self.eccentricity);
        Ok(OrbitState {
            distance_factor: ratio * ratio,
            declination_rad: declination,
            solar_longitude_rad: longitude,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EARTH_E: f64 = 0.016_710_22;
    const EARTH_PERIOD: f64 = 365.256;
    const EARTH_OBLIQUITY_DEG: f64 = 23.44;
    const EARTH_PERIHELION_LON_DEG: f64 = 102.9;

    fn earth() -> Orbit {
        Orbit {
            eccentricity: EARTH_E,
            obliquity_rad: EARTH_OBLIQUITY_DEG.to_radians(),
            perihelion_lon_rad: EARTH_PERIHELION_LON_DEG.to_radians(),
            period_days: EARTH_PERIOD,
        }
    }

    #[test]
    fn kepler_residual_below_tolerance() {
        for e in [0.0, 0.0935, 0.2056, 0.6] {
            for step in 0..360 {
                let mean = f64::from(step).to_radians();
                let big_e = eccentric_anomaly(mean, e).unwrap();
                let residual = big_e - e * big_e.sin() - mean.rem_euclid(TAU);
                assert!(residual.abs() < 1e-11, "e={e} M={mean}: residu {residual}");
            }
        }
    }

    #[test]
    fn kepler_circular_orbit_is_identity() {
        let big_e = eccentric_anomaly(1.2, 0.0).unwrap();
        assert!((big_e - 1.2).abs() < 1e-12);
    }

    #[test]
    fn true_anomaly_zero_at_perihelion() {
        assert!(true_anomaly(0.0, EARTH_E).abs() < 1e-12);
        assert!((true_anomaly(PI, EARTH_E) - PI).abs() < 1e-12);
    }

    #[test]
    fn perihelion_state_geometry() {
        let state = earth().state_at(PERIHELION_DAY).unwrap();
        let expected_lon = (EARTH_PERIHELION_LON_DEG + 180.0).to_radians();
        assert!((state.solar_longitude_rad - expected_lon).abs() < 1e-9);
        let expected_factor = 1.0 / (1.0 - EARTH_E).powi(2);
        assert!((state.distance_factor - expected_factor).abs() < 1e-9);
    }

    #[test]
    fn declination_bounded_by_obliquity() {
        let orbit = earth();
        let max = (0..365)
            .map(|d| orbit.state_at(f64::from(d)).unwrap().declination_rad.abs())
            .fold(0.0_f64, f64::max);
        assert!(max <= orbit.obliquity_rad + 0.01_f64.to_radians());
        assert!(max >= orbit.obliquity_rad - 0.01_f64.to_radians());
    }
}
