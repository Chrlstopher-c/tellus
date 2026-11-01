//! Insolation journalière moyenne au sommet de l'atmosphère (Berger 1978) et table latitude × mois.

use super::body::Body;
use super::orbit::{Orbit, OrbitError, OrbitState};
use std::f64::consts::PI;

/// Nombre de latitudes de la table : −90° à +90° au pas de 1°.
pub const LAT_STEPS: usize = 181;
/// Nombre de mois de l'année tropicale de la table.
pub const MONTHS: usize = 12;
/// Sous-échantillons par mois (règle du point milieu) pour la moyenne mensuelle.
const SAMPLES_PER_MONTH: usize = 32;

/// Insolation journalière moyenne au TOA, W/m², pour une latitude et une déclinaison (rad).
/// `s0` est le flux à distance moyenne ; `distance_factor` = (a/r)².
pub fn daily_mean_toa(s0: f64, distance_factor: f64, lat_rad: f64, decl_rad: f64) -> f64 {
    let h0 = half_day_angle(lat_rad, decl_rad);
    let daily = h0 * lat_rad.sin() * decl_rad.sin() + lat_rad.cos() * decl_rad.cos() * h0.sin();
    s0 / PI * distance_factor * daily
}

/// Demi-arc diurne H0 ; π en jour polaire, 0 en nuit polaire.
fn half_day_angle(lat_rad: f64, decl_rad: f64) -> f64 {
    let cosine = -lat_rad.tan() * decl_rad.tan();
    if cosine <= -1.0 {
        PI
    } else if cosine >= 1.0 {
        0.0
    } else {
        cosine.acos()
    }
}

/// Table Q̄[latitude][mois] en W/m² ; la ligne i correspond à la latitude i − 90°.
#[derive(Debug, Clone, PartialEq)]
pub struct InsolationTable {
    rows: Vec<[f64; MONTHS]>,
}

impl InsolationTable {
    pub fn rows(&self) -> &[[f64; MONTHS]] {
        &self.rows
    }

    /// Valeur de la latitude la plus proche (degrés) pour un mois (0 = premier mois de l'année).
    pub fn at(&self, lat_deg: f64, month: usize) -> f64 {
        let index = (lat_deg + 90.0).round().clamp(0.0, (LAT_STEPS - 1) as f64) as usize;
        self.rows[index][month % MONTHS]
    }

    /// Moyenne annuelle globale, pondérée par cos(latitude) (aire élémentaire sur la sphère).
    pub fn global_annual_mean(&self) -> f64 {
        let (weighted, weights) = self.rows.iter().enumerate().fold((0.0, 0.0), |(sum, wsum), (i, row)| {
            let weight = (i as f64 - 90.0).to_radians().cos();
            (sum + weight * annual_mean(row), wsum + weight)
        });
        weighted / weights
    }
}

fn annual_mean(row: &[f64; MONTHS]) -> f64 {
    row.iter().sum::<f64>() / MONTHS as f64
}

/// Précalcule la table latitude × mois pour un corps : moyenne mensuelle sur l'orbite képlérienne.
pub fn build_table(body: &Body) -> Result<InsolationTable, OrbitError> {
    let orbit = body.orbit();
    let month_days = orbit.period_days / MONTHS as f64;
    let mut rows = vec![[0.0; MONTHS]; LAT_STEPS];
    for month in 0..MONTHS {
        let start = month as f64 * month_days;
        let states = month_states(&orbit, start, month_days)?;
        for (i, row) in rows.iter_mut().enumerate() {
            let lat = (i as f64 - 90.0).to_radians();
            row[month] = mean_over(&states, |s| {
                daily_mean_toa(body.solar_flux_wm2, s.distance_factor, lat, s.declination_rad)
            });
        }
    }
    Ok(InsolationTable { rows })
}

fn month_states(orbit: &Orbit, start: f64, length: f64) -> Result<Vec<OrbitState>, OrbitError> {
    (0..SAMPLES_PER_MONTH)
        .map(|k| {
            let day = start + (k as f64 + 0.5) * length / SAMPLES_PER_MONTH as f64;
            orbit.state_at(day)
        })
        .collect()
}

fn mean_over(states: &[OrbitState], value: impl Fn(&OrbitState) -> f64) -> f64 {
    states.iter().map(value).sum::<f64>() / states.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::climate::body::body;
    use crate::climate::orbit::{Orbit, PERIHELION_DAY};
    use std::f64::consts::FRAC_PI_2;

    const S0_EARTH: f64 = 1361.0;

    #[test]
    fn v01_equator_equinox() {
        let q = daily_mean_toa(S0_EARTH, 1.0, 0.0, 0.0);
        assert!((q - 433.0).abs() <= 1.0, "{q}");
    }

    #[test]
    fn v02_pole_solstice_and_winter() {
        let obliquity = 23.44_f64.to_radians();
        let summer = daily_mean_toa(S0_EARTH, 1.0, FRAC_PI_2, obliquity);
        assert!((summer - 542.0).abs() <= 1.0, "{summer}");
        let winter = daily_mean_toa(S0_EARTH, 1.0, FRAC_PI_2, -obliquity);
        assert!(winter.abs() < 1e-9, "{winter}");
    }

    #[test]
    fn v03_annual_global_mean() {
        let earth = body("earth").unwrap();
        let table = build_table(&earth).unwrap();
        let mean = table.global_annual_mean();
        assert!((mean - 340.0).abs() <= 1.0, "{mean}");
    }

    #[test]
    fn v04_perihelion_aphelion_ratio() {
        let earth = body("earth").unwrap();
        let orbit: Orbit = earth.orbit();
        let peri = orbit.state_at(PERIHELION_DAY).unwrap().distance_factor;
        let aph = orbit
            .state_at(PERIHELION_DAY + earth.orbital_period_days / 2.0)
            .unwrap()
            .distance_factor;
        assert!((peri / aph - 1.069).abs() <= 0.001, "{}", peri / aph);
    }

    #[test]
    fn v05_declination_within_obliquity() {
        let earth = body("earth").unwrap();
        let orbit = earth.orbit();
        let max = (0..365)
            .map(|d| orbit.state_at(f64::from(d)).unwrap().declination_rad.abs())
            .fold(0.0_f64, f64::max);
        assert!(max <= orbit.obliquity_rad + 0.01_f64.to_radians());
    }

    #[test]
    fn table_shape_and_polar_regimes() {
        let earth = body("earth").unwrap();
        let table = build_table(&earth).unwrap();
        assert_eq!(table.rows().len(), LAT_STEPS);
        let north_pole_june = table.at(90.0, 5);
        let south_pole_june = table.at(-90.0, 5);
        assert!(
            north_pole_june > 400.0 && south_pole_june < 1.0,
            "{north_pole_june} {south_pole_june}"
        );
    }
}
