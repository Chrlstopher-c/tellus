//! Genèse d'une planète : pipeline pur G1→G8 (plaques → mouvement → frontières → profils → âge et profondeur
//! → relief → érosion → niveau de la mer → contrôles). Chaque étape est déterministe pour (graine, grille) ;
//! tout aléa passe par `shared::rng`. La tectonique est figée après genèse (MVP).

mod g1_plates;
mod g2_motion;
mod g3_boundaries;
mod g4_profiles;
mod g5_depth;
mod g6_erosion;
mod g6_relief;
mod g7_sea_level;
mod g8_checks;
mod graph;
mod noise;
mod presets;
mod vec3;

#[cfg(test)]
mod tests;

use std::fmt;

pub use g2_motion::Plate;
pub use presets::real_body;

use crate::shared::grid::Grid;
use crate::shared::units::R_EARTH_M;

const WATER_FRACTION_DEFAULT: f64 = 0.708;
const LEVEL_DEFAULT: u8 = 7;

/// Paramètres d'un monde : graine, niveau de grille (doit correspondre à la grille), rayon, fraction d'eau.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldParams {
    pub seed: u64,
    pub level: u8,
    pub radius_m: f64,
    pub water_fraction: f64,
}

impl Default for WorldParams {
    fn default() -> Self {
        Self {
            seed: 0,
            level: LEVEL_DEFAULT,
            radius_m: R_EARTH_M,
            water_fraction: WATER_FRACTION_DEFAULT,
        }
    }
}

/// Planète générée : champs par cellule (SoA, f32) et table des plaques.
/// `crust` : 0 = océanique, 1 = continentale. `age_ma` : 0 sur les continents.
#[derive(Debug, Clone, PartialEq)]
pub struct Planet {
    pub elevation_m: Vec<f32>,
    pub crust: Vec<f32>,
    pub plate: Vec<f32>,
    pub age_ma: Vec<f32>,
    pub plates: Vec<Plate>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GenesisError {
    LevelMismatch { expected: u8, got: u8 },
    InvalidParam(&'static str),
    GridDisconnected,
    NoBoundary,
    Calibration { target_m: f64 },
    NonFinite { field: &'static str, cell: usize },
    Inconsistent(&'static str),
}

impl fmt::Display for GenesisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LevelMismatch { expected, got } => {
                write!(f, "niveau {got} demandé, grille de niveau {expected}")
            }
            Self::InvalidParam(what) => write!(f, "paramètre invalide : {what}"),
            Self::GridDisconnected => write!(f, "grille non connexe"),
            Self::NoBoundary => write!(f, "aucune frontière de plaques"),
            Self::Calibration { target_m } => write!(f, "cible d'altitude moyenne {target_m} m inatteignable"),
            Self::NonFinite { field, cell } => write!(f, "valeur non finie dans {field}, cellule {cell}"),
            Self::Inconsistent(what) => write!(f, "incohérence : {what}"),
        }
    }
}

impl std::error::Error for GenesisError {}

/// État intermédiaire avant érosion : tout ce qui précède G6.
pub(super) struct Stage {
    pub(super) plate_of: Vec<u32>,
    pub(super) continental: Vec<bool>,
    pub(super) plates: Vec<Plate>,
    pub(super) ages_ma: Vec<f64>,
    pub(super) heights_m: Vec<f64>,
}

/// Génère une planète sur la grille fournie.
pub fn generate(grid: &Grid, params: &WorldParams) -> Result<Planet, GenesisError> {
    validate(grid, params)?;
    let stage = assemble(grid, params)?;
    let (land, areas, water) = (&stage.continental, grid.areas_m2(), params.water_fraction);
    let base_level = g7_sea_level::sea_level_m(&stage.heights_m, areas, water);
    let mut eroded = stage.heights_m.clone();
    let erosion = g6_erosion::erode(grid, land, &mut eroded, base_level);
    g8_checks::check_mass(&erosion)?;
    let alpha = g7_sea_level::alpha_for_land_mean(&eroded, land, areas, water)?;
    let scaled = g7_sea_level::scaled_continents(&eroded, land, alpha);
    let surface = g7_sea_level::shifted_to_sea(&scaled, areas, water);
    let planet = finalize(&stage, &surface);
    g8_checks::check(&planet)?;
    Ok(planet)
}

fn validate(grid: &Grid, params: &WorldParams) -> Result<(), GenesisError> {
    if grid.level() != params.level {
        return Err(GenesisError::LevelMismatch {
            expected: grid.level(),
            got: params.level,
        });
    }
    if !(params.radius_m.is_finite() && params.radius_m > 0.0) {
        return Err(GenesisError::InvalidParam("rayon"));
    }
    if (grid.radius_m() - params.radius_m).abs() > 1e-6 * params.radius_m {
        return Err(GenesisError::InvalidParam("rayon différent de celui de la grille"));
    }
    if !(params.water_fraction > 0.0 && params.water_fraction < 1.0) {
        return Err(GenesisError::InvalidParam("fraction d'eau hors ]0, 1["));
    }
    Ok(())
}

/// Étapes G1 à G5 puis relief : hauteurs avant érosion.
pub(super) fn assemble(grid: &Grid, params: &WorldParams) -> Result<Stage, GenesisError> {
    let map = g1_plates::partition(grid, params.seed)?;
    let plates = g2_motion::euler_plates(params.seed, &map.continental, params.radius_m);
    let continental: Vec<bool> = map.plate_of.iter().map(|&p| map.continental[p as usize]).collect();
    let boundaries = g3_boundaries::classify_cells(grid, &map.plate_of, &plates);
    let tectonic = g4_profiles::tectonic_field(grid, &map.plate_of, &plates, &boundaries)?;
    let ages_ma = g5_depth::ocean_ages(grid, &map.plate_of, &plates, &boundaries);
    let mut heights_m = g5_depth::base_heights(&map.plate_of, &plates, &ages_ma, &tectonic);
    g6_relief::add_relief(grid, params.seed, &map.plate_of, &plates, &tectonic, &mut heights_m);
    Ok(Stage {
        plate_of: map.plate_of,
        continental,
        plates,
        ages_ma,
        heights_m,
    })
}

fn finalize(stage: &Stage, heights: &[f64]) -> Planet {
    Planet {
        elevation_m: heights.iter().map(|&h| h as f32).collect(),
        crust: stage.continental.iter().map(|&c| if c { 1.0 } else { 0.0 }).collect(),
        plate: stage.plate_of.iter().map(|&p| p as f32).collect(),
        age_ma: stage.ages_ma.iter().map(|&a| a as f32).collect(),
        plates: stage.plates.clone(),
    }
}
