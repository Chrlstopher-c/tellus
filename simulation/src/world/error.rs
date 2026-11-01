//! Erreurs de l'orchestrateur `world` : requête invalide, domaines amont, commandes non disponibles.

use crate::climate::body::BodyError;
use crate::climate::error::ClimateError;
use crate::climate::greenhouse::GreenhouseError;
use crate::genesis::GenesisError;
use crate::matter::MatterError;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum WorldError {
    InvalidRequest(String),
    InvalidLevel(u32),
    UnknownBody(String),
    InvalidDuration(f64),
    InvalidRate(f64),
    CellOutOfRange { cell: u32, cells: usize },
    PowersUnavailable,
    NoWorld,
    Poisoned,
    Genesis(GenesisError),
    Body(BodyError),
    Greenhouse(GreenhouseError),
    Climate(ClimateError),
    Matter(MatterError),
    LifeInputs,
}

impl fmt::Display for WorldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(reason) => write!(f, "requête de monde invalide : {reason}"),
            Self::InvalidLevel(level) => write!(f, "niveau de grille {level} hors de 0..=7"),
            Self::UnknownBody(name) => write!(f, "corps inconnu : {name}"),
            Self::InvalidDuration(dt) => write!(f, "durée {dt} an(s) invalide (attendu : fini et ≥ 0)"),
            Self::InvalidRate(rate) => write!(f, "vitesse {rate} an/s hors des vitesses de jeu"),
            Self::CellOutOfRange { cell, cells } => write!(f, "cellule {cell} hors de 0..{cells}"),
            Self::PowersUnavailable => write!(f, "pouvoirs pas encore disponibles"),
            Self::NoWorld => write!(f, "aucun monde créé"),
            Self::Poisoned => write!(f, "état du monde corrompu (verrou empoisonné)"),
            Self::Genesis(error) => write!(f, "genèse : {error}"),
            Self::Body(error) => write!(f, "corps réel : {error}"),
            Self::Greenhouse(error) => write!(f, "effet de serre : {error}"),
            Self::Climate(error) => write!(f, "climat : {error}"),
            Self::Matter(error) => write!(f, "matière : {error}"),
            Self::LifeInputs => write!(f, "forçages du vivant incohérents : pas de pas"),
        }
    }
}

impl std::error::Error for WorldError {}

impl From<GenesisError> for WorldError {
    fn from(error: GenesisError) -> Self {
        Self::Genesis(error)
    }
}

impl From<BodyError> for WorldError {
    fn from(error: BodyError) -> Self {
        Self::Body(error)
    }
}

impl From<GreenhouseError> for WorldError {
    fn from(error: GreenhouseError) -> Self {
        Self::Greenhouse(error)
    }
}

impl From<ClimateError> for WorldError {
    fn from(error: ClimateError) -> Self {
        Self::Climate(error)
    }
}

impl From<MatterError> for WorldError {
    fn from(error: MatterError) -> Self {
        Self::Matter(error)
    }
}
