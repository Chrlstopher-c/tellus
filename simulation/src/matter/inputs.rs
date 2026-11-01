//! Entrées par cellule du domaine matière (contrat REGLES § 12) : climat, masque et NPP réalisée.

use super::error::MatterError;

/// Tranches par cellule empruntées aux autres domaines ; toutes de la longueur de la grille. La NPP et la biomasse
/// peuvent être nulles (planète sans vie). `biomass_gc_m2` plafonne le stock végétal de carbone ; l'excédent passe
/// en litière.
#[derive(Debug, Clone, Copy)]
pub struct MatterInputs<'a> {
    pub temperature_c: &'a [f32],
    pub precip_mm_yr: &'a [f32],
    pub insolation_w_m2: &'a [f32],
    pub ice_fraction: &'a [f32],
    pub is_ocean: &'a [bool],
    pub elevation_m: &'a [f32],
    pub npp_realized_gc_m2_yr: &'a [f32],
    pub biomass_gc_m2: &'a [f32],
}

fn finite(value: f32) -> bool {
    value.is_finite()
}

fn non_negative(value: f32) -> bool {
    value.is_finite() && value >= 0.0
}

fn unit_interval(value: f32) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

fn check(field: &'static str, values: &[f32], valid: fn(f32) -> bool) -> Result<(), MatterError> {
    match values.iter().position(|&v| !valid(v)) {
        Some(index) if !values[index].is_finite() => Err(MatterError::NonFinite { field, index }),
        Some(index) => Err(MatterError::OutOfRange { field, index }),
        None => Ok(()),
    }
}

impl MatterInputs<'_> {
    /// Vérifie les longueurs puis les valeurs : une entrée invalide arrête le tick avant toute écriture.
    pub(crate) fn validate(&self, cells: usize) -> Result<(), MatterError> {
        let lengths = [
            ("temperature_c", self.temperature_c.len()),
            ("precip_mm_yr", self.precip_mm_yr.len()),
            ("insolation_w_m2", self.insolation_w_m2.len()),
            ("ice_fraction", self.ice_fraction.len()),
            ("is_ocean", self.is_ocean.len()),
            ("elevation_m", self.elevation_m.len()),
            ("npp_realized_gc_m2_yr", self.npp_realized_gc_m2_yr.len()),
            ("biomass_gc_m2", self.biomass_gc_m2.len()),
        ];
        for (field, got) in lengths {
            if got != cells {
                return Err(MatterError::LengthMismatch {
                    field,
                    expected: cells,
                    got,
                });
            }
        }
        check("temperature_c", self.temperature_c, finite)?;
        check("precip_mm_yr", self.precip_mm_yr, non_negative)?;
        check("insolation_w_m2", self.insolation_w_m2, non_negative)?;
        check("ice_fraction", self.ice_fraction, unit_interval)?;
        check("elevation_m", self.elevation_m, finite)?;
        check("npp_realized_gc_m2_yr", self.npp_realized_gc_m2_yr, non_negative)?;
        check("biomass_gc_m2", self.biomass_gc_m2, non_negative)
    }
}
