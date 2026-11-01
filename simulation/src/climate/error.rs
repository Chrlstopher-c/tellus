//! Erreurs du climat : entrées incohérentes, régimes, bornes numériques et erreurs des domaines amont.

use super::greenhouse::GreenhouseError;
use super::mesh::MeshError;
use super::orbit::OrbitError;
use crate::clock::Regime;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ClimateError {
    InputLength {
        field: &'static str,
        expected: usize,
        got: usize,
    },
    InvalidDuration(f64),
    RegimeMismatch {
        given: Regime,
        expected: Regime,
    },
    NonFinite {
        field: &'static str,
        cell: usize,
    },
    FrozenRegime,
    Mesh(MeshError),
    Greenhouse(GreenhouseError),
    Orbit(OrbitError),
}

impl fmt::Display for ClimateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputLength { field, expected, got } => {
                write!(f, "champ '{field}' : {got} valeurs, {expected} attendues")
            }
            Self::InvalidDuration(dt) => write!(f, "durée de pas invalide : {dt} an"),
            Self::RegimeMismatch { given, expected } => {
                write!(f, "régime {given:?} fourni pour un Δt de régime {expected:?}")
            }
            Self::NonFinite { field, cell } => write!(f, "valeur non finie dans '{field}' cellule {cell}"),
            Self::FrozenRegime => write!(f, "régime 'figé' : le climat n'a pas de régime figé"),
            Self::Mesh(err) => write!(f, "maillage : {err}"),
            Self::Greenhouse(err) => write!(f, "effet de serre : {err}"),
            Self::Orbit(err) => write!(f, "orbite : {err}"),
        }
    }
}

impl std::error::Error for ClimateError {}

impl From<MeshError> for ClimateError {
    fn from(err: MeshError) -> Self {
        Self::Mesh(err)
    }
}

impl From<GreenhouseError> for ClimateError {
    fn from(err: GreenhouseError) -> Self {
        Self::Greenhouse(err)
    }
}

impl From<OrbitError> for ClimateError {
    fn from(err: OrbitError) -> Self {
        Self::Orbit(err)
    }
}
