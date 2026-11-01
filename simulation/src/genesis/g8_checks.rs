//! G8 : contrôles finaux. Une planète non finie ou incohérente n'est jamais rendue, même partiellement.

use super::g6_erosion::ErosionReport;
use super::{GenesisError, Planet};

const MASS_TOLERANCE: f64 = 1e-6;

/// La matière érodée doit être entièrement déposée sur l'océan (Σ érodé = Σ déposé).
pub(super) fn check_mass(report: &ErosionReport) -> Result<(), GenesisError> {
    let gap = (report.removed_m3 - report.deposited_m3).abs();
    if gap > MASS_TOLERANCE * report.removed_m3.abs().max(1.0) {
        return Err(GenesisError::Inconsistent("érosion non conservative"));
    }
    Ok(())
}

pub(super) fn check(planet: &Planet) -> Result<(), GenesisError> {
    check_finite("elevation_m", &planet.elevation_m)?;
    check_finite("crust", &planet.crust)?;
    check_finite("plate", &planet.plate)?;
    check_finite("age_ma", &planet.age_ma)?;
    check_plate_ids(planet)?;
    check_ages(planet)
}

fn check_finite(field: &'static str, values: &[f32]) -> Result<(), GenesisError> {
    match values.iter().position(|v| !v.is_finite()) {
        Some(cell) => Err(GenesisError::NonFinite { field, cell }),
        None => Ok(()),
    }
}

fn check_plate_ids(planet: &Planet) -> Result<(), GenesisError> {
    let count = planet.plates.len();
    if planet.plate.iter().any(|&p| p < 0.0 || p as usize >= count) {
        return Err(GenesisError::Inconsistent("identifiant de plaque hors table"));
    }
    Ok(())
}

fn check_ages(planet: &Planet) -> Result<(), GenesisError> {
    if planet.age_ma.iter().any(|&a| a < 0.0) {
        return Err(GenesisError::Inconsistent("âge océanique négatif"));
    }
    Ok(())
}
