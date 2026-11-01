//! Erreurs du domaine matière : entrées invalides refusées avant tout calcul (échec bruyant, REGLES § 0.6).

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum MatterError {
    InvalidTimeStep(f64),
    InvalidConfig(&'static str),
    LengthMismatch {
        field: &'static str,
        expected: usize,
        got: usize,
    },
    NonFinite {
        field: &'static str,
        index: usize,
    },
    OutOfRange {
        field: &'static str,
        index: usize,
    },
    MaskChanged,
}

impl fmt::Display for MatterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTimeStep(dt) => write!(f, "pas de temps invalide : {dt} an"),
            Self::InvalidConfig(reason) => write!(f, "configuration matière invalide : {reason}"),
            Self::LengthMismatch { field, expected, got } => {
                write!(f, "'{field}' : {got} valeurs, {expected} cellules attendues")
            }
            Self::NonFinite { field, index } => write!(f, "'{field}' : valeur non finie à la cellule {index}"),
            Self::OutOfRange { field, index } => write!(f, "'{field}' : valeur hors bornes à la cellule {index}"),
            Self::MaskChanged => write!(f, "masque océan-continent modifié en cours de simulation"),
        }
    }
}

impl std::error::Error for MatterError {}
