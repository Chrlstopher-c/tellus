//! Extinction de lignée : effectif total sous le seuil pendant une durée continue.

use super::lineage::{LineageId, LineageTable};

/// Effectif total sous lequel une lignée est en déclin (individus).
pub const MIN_POPULATION: f64 = 10.0;
/// Durée continue sous le seuil avant extinction, années (vivant.md § 6.5).
pub const EXTINCTION_YEARS: f64 = 50.0;

/// Met à jour le compteur de déclin des lignées vivantes (effectifs déjà rafraîchis) ; renvoie les éteintes.
pub fn update_extinctions(table: &mut LineageTable, check_years: f64, tick: u64) -> Vec<LineageId> {
    let mut extinct = Vec::new();
    for id in table.alive_ids() {
        let Some(lineage) = table.get_mut(id) else {
            continue;
        };
        if lineage.n_total < MIN_POPULATION {
            lineage.low_yr += check_years;
        } else {
            lineage.low_yr = 0.0;
        }
        if lineage.low_yr >= EXTINCTION_YEARS {
            lineage.extinct_tick = Some(tick);
            extinct.push(id);
        }
    }
    extinct
}
