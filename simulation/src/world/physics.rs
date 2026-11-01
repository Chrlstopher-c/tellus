//! Paramètres physiques d'un monde : rayon, gravité, flux solaire, et corps réels par nom d'affichage.

use super::error::WorldError;
use super::request::GeneratedSpec;
use crate::climate::body::{self, Body};
use crate::shared::units::{G_STANDARD, R_EARTH_M};

/// Constante solaire terrestre, W/m².
const SOLAR_CONSTANT_EARTH_W_M2: f64 = 1_361.0;
/// Corps réels jouables : nom affiché dans l'interface → identifiant de `simulation/data/planets.json`.
const PLAYABLE_BODIES: [(&str, &str); 6] = [
    ("Mercure", "mercury"),
    ("Vénus", "venus"),
    ("Venus", "venus"),
    ("Terre", "earth"),
    ("Lune", "moon"),
    ("Mars", "mars"),
];

/// Grandeurs dérivées d'un monde généré depuis `WorldConfig`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeneratedPhysics {
    pub radius_m: f64,
    pub gravity_ms2: f64,
    pub solar_flux_wm2: f64,
    pub water_fraction: f64,
}

impl GeneratedPhysics {
    /// Rayon = 6 371 km × taille ; gravité à densité moyenne terrestre (∝ rayon) ; S = 1361 / (distance)².
    pub fn from_spec(spec: &GeneratedSpec) -> Self {
        let size = spec.size_percent / 100.0;
        let distance = spec.distance_percent / 100.0;
        Self {
            radius_m: R_EARTH_M * size,
            gravity_ms2: G_STANDARD * size,
            solar_flux_wm2: SOLAR_CONSTANT_EARTH_W_M2 / (distance * distance),
            water_fraction: spec.water_percent / 100.0,
        }
    }
}

/// Identifiant `planets.json` d'un corps jouable, désigné par son nom affiché ou son identifiant.
pub fn playable_body_id(name: &str) -> Result<&'static str, WorldError> {
    PLAYABLE_BODIES
        .iter()
        .find(|(display, id)| *display == name || *id == name)
        .map(|(_, id)| *id)
        .ok_or_else(|| WorldError::UnknownBody(name.to_owned()))
}

/// Gabarit « Terre » du monde généré (orbite, pression, albédo, période), avec son flux solaire et son rayon propres.
pub fn generated_body(name: &str, physics: &GeneratedPhysics) -> Result<Body, WorldError> {
    let mut body = body::body("earth")?;
    body.id = "generated".to_owned();
    body.name_fr = name.to_owned();
    body.radius_km = physics.radius_m / 1_000.0;
    body.surface_gravity_ms2 = physics.gravity_ms2;
    body.solar_flux_wm2 = physics.solar_flux_wm2;
    Ok(body)
}
