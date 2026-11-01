//! État du monde côté pont : création, avancée du temps, instantané d'altitude. Sans type napi.

use crate::shared::grid::Grid;
use crate::shared::units::R_EARTH_M;
use std::fmt;
use std::sync::{Mutex, MutexGuard};

pub const MAX_LEVEL: u32 = 7;

#[derive(Debug, Clone, PartialEq)]
pub enum WorldError {
    InvalidLevel(u32),
    InvalidDuration(f64),
    NoWorld,
    Poisoned,
}

impl fmt::Display for WorldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLevel(level) => write!(f, "niveau de grille {level} hors de 0..={MAX_LEVEL}"),
            Self::InvalidDuration(dt) => write!(f, "durée {dt} an(s) invalide (attendu : fini et ≥ 0)"),
            Self::NoWorld => write!(f, "aucun monde créé"),
            Self::Poisoned => write!(f, "état du monde corrompu (verrou empoisonné)"),
        }
    }
}

impl std::error::Error for WorldError {}

pub struct World {
    seed: u32,
    grid: Grid,
    elevation_m: Vec<f32>,
    time_years: f64,
}

impl World {
    pub fn new(seed: u32, level: u32) -> Result<Self, WorldError> {
        if level > MAX_LEVEL {
            return Err(WorldError::InvalidLevel(level));
        }
        let level = u8::try_from(level).map_err(|_| WorldError::InvalidLevel(level))?;
        let grid = Grid::new(level, R_EARTH_M);
        let elevation_m = vec![0.0; grid.cell_count()];
        Ok(Self {
            seed,
            grid,
            elevation_m,
            time_years: 0.0,
        })
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn cell_count(&self) -> usize {
        self.grid.cell_count()
    }

    pub fn advance(&mut self, dt_years: f64) -> Result<f64, WorldError> {
        if !dt_years.is_finite() || dt_years < 0.0 {
            return Err(WorldError::InvalidDuration(dt_years));
        }
        self.time_years += dt_years;
        Ok(self.time_years)
    }

    pub fn elevation_snapshot(&self) -> Vec<f32> {
        self.elevation_m.clone()
    }
}

/// Emplacement unique du monde courant, protégé par un verrou.
pub struct WorldSlot {
    inner: Mutex<Option<World>>,
}

impl WorldSlot {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    /// Remplace le monde courant et renvoie son nombre de cellules.
    pub fn create(&self, seed: u32, level: u32) -> Result<usize, WorldError> {
        let world = World::new(seed, level)?;
        let cells = world.cell_count();
        *self.lock()? = Some(world);
        Ok(cells)
    }

    /// Avance le monde de `dt_years` et renvoie le temps simulé total.
    pub fn step(&self, dt_years: f64) -> Result<f64, WorldError> {
        let mut guard = self.lock()?;
        guard.as_mut().ok_or(WorldError::NoWorld)?.advance(dt_years)
    }

    pub fn snapshot(&self) -> Result<Vec<f32>, WorldError> {
        let guard = self.lock()?;
        guard.as_ref().map(World::elevation_snapshot).ok_or(WorldError::NoWorld)
    }

    fn lock(&self) -> Result<MutexGuard<'_, Option<World>>, WorldError> {
        self.inner.lock().map_err(|_| WorldError::Poisoned)
    }
}

impl Default for WorldSlot {
    fn default() -> Self {
        Self::new()
    }
}
