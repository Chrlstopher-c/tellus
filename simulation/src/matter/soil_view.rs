//! Lecture seule de l'état matière pour l'orchestrateur (inspecteur, constantes de vie) : sol de la cellule normalisé
//! 0-1 et fraction d'O₂. Aucune écriture : les stocks restent sous le registre `shared::ledger`.

use super::carbon;
use super::oxygen;
use super::state::{per_area_g_m2, MatterState};
use super::water;

/// Carbone de sol (litière active + humus) qui sature l'indicateur de l'inspecteur, g m⁻² (calage, ≈ 20 kg m⁻²).
const SOIL_CARBON_SCALE_G_M2: f64 = 20_000.0;

/// Stocks de sol d'une cellule, normalisés 0-1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoilView {
    pub water: f64,
    pub carbon: f64,
    pub nitrogen: f64,
    pub phosphorus: f64,
}

impl MatterState {
    /// Sol de la cellule : eau sur capacité au champ, carbone organique saturé, azote et phosphore disponibles.
    pub fn soil(&self, cell: usize) -> SoilView {
        let area = self.geometry.areas_m2[cell];
        let stored_water = self.water.get(self.water.layout.at(water::SOIL, cell));
        let soil_carbon_gt = self.carbon.get(self.carbon.layout.at(carbon::FAST, cell))
            + self.carbon.get(self.carbon.layout.at(carbon::SLOW, cell));
        SoilView {
            water: ratio(stored_water, water::soil_capacity_km3(area)),
            carbon: ratio(per_area_g_m2(soil_carbon_gt, area), SOIL_CARBON_SCALE_G_M2),
            nitrogen: f64::from(self.n_avail[cell]).clamp(0.0, 1.0),
            phosphorus: f64::from(self.p_avail[cell]).clamp(0.0, 1.0),
        }
    }

    /// Fraction molaire d'O₂ atmosphérique (0 à 1).
    pub fn o2_fraction(&self) -> f64 {
        oxygen::fraction(self.o2_tmol())
    }
}

fn ratio(value: f64, scale: f64) -> f64 {
    if scale > 0.0 {
        (value / scale).clamp(0.0, 1.0)
    } else {
        0.0
    }
}
