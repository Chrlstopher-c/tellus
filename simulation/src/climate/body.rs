//! Corps réels : chargement de `simulation/data/planets.json` (sources NASA) et paramètres dérivés.

use super::orbit::Orbit;
use crate::shared::units::SIGMA;
use serde_json::Value;
use std::fmt;

const PLANETS_JSON: &str = include_str!("../../data/planets.json");
/// Longitude du périhélie terrestre (102,9°), valeur de repli quand le JSON ne la fournit pas.
pub const EARTH_PERIHELION_LON_DEG: f64 = 102.9;

/// Corps réel, valeurs SI ou degrés/heures/jours selon le champ.
#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    pub id: String,
    pub name_fr: String,
    pub radius_km: f64,
    pub surface_gravity_ms2: f64,
    /// Flux solaire reçu à la distance moyenne du corps, W/m².
    pub solar_flux_wm2: f64,
    pub bond_albedo: f64,
    pub eccentricity: f64,
    pub obliquity_deg: f64,
    pub perihelion_lon_deg: f64,
    pub orbital_period_days: f64,
    pub sidereal_rotation_hours: f64,
    pub solar_day_hours: f64,
    /// Température moyenne de surface observée, K (cible de calibration de l'effet de serre).
    pub surface_temp_mean_k: Option<f64>,
    /// Pression de surface, bar (entrée de la loi τ(P, CO₂) des mondes générés).
    pub surface_pressure_bar: Option<f64>,
    /// Fraction molaire de CO₂ de l'atmosphère (référence de la loi de CO₂), sans dimension.
    pub co2_fraction: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BodyError {
    UnknownBody(String),
    MissingField { body: String, field: &'static str },
    Catalog(String),
}

impl fmt::Display for BodyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownBody(id) => write!(f, "corps inconnu : {id}"),
            Self::MissingField { body, field } => write!(f, "corps '{body}' : champ '{field}' absent"),
            Self::Catalog(reason) => write!(f, "planets.json illisible : {reason}"),
        }
    }
}

impl std::error::Error for BodyError {}

impl Body {
    /// Paramètres orbitaux, angles convertis en radians.
    pub fn orbit(&self) -> Orbit {
        Orbit {
            eccentricity: self.eccentricity,
            obliquity_rad: self.obliquity_deg.to_radians(),
            perihelion_lon_rad: self.perihelion_lon_deg.to_radians(),
            period_days: self.orbital_period_days,
        }
    }

    /// Température d'équilibre sans effet de serre (S(1 − A) / 4σ)^(1/4), K.
    pub fn equilibrium_temperature_k(&self) -> f64 {
        (self.solar_flux_wm2 * (1.0 - self.bond_albedo) / (4.0 * SIGMA)).powf(0.25)
    }

    fn from_entry(entry: &Value) -> Result<Self, BodyError> {
        let id = entry
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| BodyError::Catalog("entrée sans identifiant".to_owned()))?;
        Ok(Self {
            id: id.to_owned(),
            name_fr: entry.get("name_fr").and_then(Value::as_str).unwrap_or(id).to_owned(),
            radius_km: number(entry, id, "radius_km")?,
            surface_gravity_ms2: number(entry, id, "surface_gravity_ms2")?,
            solar_flux_wm2: number(entry, id, "solar_flux_wm2")?,
            bond_albedo: number(entry, id, "bond_albedo")?,
            eccentricity: number(entry, id, "eccentricity")?,
            obliquity_deg: number(entry, id, "obliquity_deg")?,
            perihelion_lon_deg: entry
                .get("perihelion_lon_deg")
                .and_then(Value::as_f64)
                .unwrap_or(EARTH_PERIHELION_LON_DEG),
            orbital_period_days: number(entry, id, "orbital_period_days")?,
            sidereal_rotation_hours: number(entry, id, "sidereal_rotation_hours")?,
            solar_day_hours: number(entry, id, "solar_day_hours")?,
            surface_temp_mean_k: entry.get("surface_temp_mean_k").and_then(Value::as_f64),
            surface_pressure_bar: entry.get("surface_pressure_bar").and_then(Value::as_f64),
            co2_fraction: entry
                .get("atmosphere_molar_fractions")
                .and_then(|fractions| fractions.get("CO2"))
                .and_then(Value::as_f64),
        })
    }
}

fn number(entry: &Value, id: &str, field: &'static str) -> Result<f64, BodyError> {
    entry
        .get(field)
        .and_then(Value::as_f64)
        .ok_or_else(|| BodyError::MissingField {
            body: id.to_owned(),
            field,
        })
}

fn parse_catalog(source: &str) -> Result<Vec<Value>, BodyError> {
    match serde_json::from_str::<Value>(source) {
        Ok(Value::Array(entries)) => Ok(entries),
        Ok(_) => Err(BodyError::Catalog("racine non tableau".to_owned())),
        Err(err) => Err(BodyError::Catalog(err.to_string())),
    }
}

/// Toutes les entrées du catalogue : chacune est un corps chargé ou une erreur explicite.
pub fn catalog() -> Result<Vec<Result<Body, BodyError>>, BodyError> {
    Ok(parse_catalog(PLANETS_JSON)?.iter().map(Body::from_entry).collect())
}

/// Corps par identifiant (`earth`, `mars`, …) depuis `simulation/data/planets.json`.
pub fn body(id: &str) -> Result<Body, BodyError> {
    let entries = parse_catalog(PLANETS_JSON)?;
    let entry = entries
        .iter()
        .find(|entry| entry.get("id").and_then(Value::as_str) == Some(id))
        .ok_or_else(|| BodyError::UnknownBody(id.to_owned()))?;
    Body::from_entry(entry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_teq() {
        let earth = body("earth").unwrap();
        let teq = earth.equilibrium_temperature_k();
        assert!((teq - 255.0).abs() <= 1.0, "{teq}");
    }

    #[test]
    fn all_bodies_load() {
        let entries = catalog().unwrap();
        assert_eq!(entries.len(), 11);
        let failed: Vec<String> = entries
            .iter()
            .filter_map(|entry| entry.as_ref().err().map(ToString::to_string))
            .collect();
        assert_eq!(failed.len(), 2, "{failed:?}");
        assert!(failed.iter().any(|e| e.contains("titan")));
        assert!(failed.iter().any(|e| e.contains("europa")));
    }

    #[test]
    fn unknown_body_is_explicit_error() {
        assert!(matches!(body("pluton"), Err(BodyError::UnknownBody(_))));
    }

    #[test]
    fn missing_perihelion_defaults_to_earth() {
        let mercury = body("mercury").unwrap();
        assert!((mercury.perihelion_lon_deg - 102.9).abs() < 1e-9);
    }
}
