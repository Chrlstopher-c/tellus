//! Cycle de l'azote (nutriments § 3) : végétation, sol organique, azote minéral par cellule terrestre ; nitrate
//! océanique par cellule. Fixation couvrant le déficit, dénitrification, lessivage vers les rivières puis l'océan.

use super::forcing::{Forcing, Scratch};
use super::params::{
    CN_OCEAN, CN_VEG, DENITRIFICATION_PER_YR, EXPORT_RATIO, FIX_CAP_GN_M2_YR, INIT_DEEP_N_GTN, INIT_DIN_G_M2,
    LEACH_N_PER_YR, N2_RESERVE_GTN, TAU_UPWELL_N_YR,
};
use super::state::{gt_from_g_m2, Element, Geometry, Layout};
use crate::shared::units::GRAMS_PER_GT;

pub(crate) const VEG: usize = 0;
pub(crate) const SOIL: usize = 1;
pub(crate) const MIN: usize = 2;
pub(crate) const OCEAN: usize = 3;
const KINDS: usize = 4;
pub(crate) const N2: usize = 0;
pub(crate) const DEEP: usize = 1;
pub(crate) const RIVER: usize = 2;
const GLOBALS: usize = 3;

pub(crate) fn initial(geo: &Geometry) -> Element {
    let layout = Layout {
        cells: geo.cell_count(),
        kinds: KINDS,
        globals: GLOBALS,
    };
    let mut stocks = vec![0.0; layout.len()];
    stocks[layout.glob(N2)] = N2_RESERVE_GTN - INIT_DEEP_N_GTN;
    stocks[layout.glob(DEEP)] = INIT_DEEP_N_GTN;
    for (i, &area) in geo.areas_m2.iter().enumerate() {
        if geo.is_ocean[i] {
            stocks[layout.at(OCEAN, i)] = gt_from_g_m2(INIT_DIN_G_M2, area);
        }
    }
    Element::from_stocks(layout, stocks)
}

fn land_cell(n: &mut Element, area_m2: f64, i: usize, forcing: &Forcing, scratch: &Scratch, dt: f64) {
    let lay = n.layout;
    let (min, veg, soil) = (lay.at(MIN, i), lay.at(VEG, i), lay.at(SOIL, i));
    let (n2, river) = (lay.glob(N2), lay.glob(RIVER));
    let demand = forcing.npp_eff[i] * area_m2 / GRAMS_PER_GT * dt / CN_VEG;
    let mineralised = (forcing.soil_turnover[i] * dt).min(1.0) * n.get(soil);
    n.flow(soil, min, mineralised);
    let fixation_cap = FIX_CAP_GN_M2_YR * area_m2 / GRAMS_PER_GT * dt;
    let deficit = (demand - n.get(min)).max(0.0);
    n.flow(n2, min, deficit.min(fixation_cap));
    n.flow(min, veg, demand);
    n.flow(veg, soil, scratch.litter_gtc[i] / CN_VEG);
    let denitrified = (1.0 - (-DENITRIFICATION_PER_YR * dt).exp()) * n.get(min);
    n.flow(min, n2, denitrified);
    let leached = (1.0 - (-LEACH_N_PER_YR * scratch.runoff_fraction[i] * dt).exp()) * n.get(min);
    n.flow(min, river, leached);
}

/// Sous-pas terrestre de l'azote : fixation du déficit, prélèvement, litière, minéralisation, pertes.
pub(crate) fn land_substep(n: &mut Element, geo: &Geometry, forcing: &Forcing, scratch: &Scratch, dt: f64) {
    for i in geo.land_cells() {
        land_cell(n, geo.areas_m2[i], i, forcing, scratch, dt);
    }
}

/// Export d'azote organique marin vers le fond, au rapport Redfield (C:N ≈ 5,68 en masse).
pub(crate) fn marine_export(n: &mut Element, geo: &Geometry, forcing: &Forcing, dt: f64) {
    let deep = n.layout.glob(DEEP);
    for i in geo.ocean_cells() {
        let export = EXPORT_RATIO * forcing.npp_eff[i] * geo.areas_m2[i] / GRAMS_PER_GT * dt / CN_OCEAN;
        let source = n.layout.at(OCEAN, i);
        n.flow(source, deep, export);
    }
}

/// Transport de fin de tick : rivière vers l'océan, remontée de l'azote profond.
pub(crate) fn transport(n: &mut Element, geo: &Geometry, dt: f64) {
    let (river, deep) = (n.layout.glob(RIVER), n.layout.glob(DEEP));
    let river_total = n.get(river);
    let upwelling = n.get(deep) * (1.0 - (-dt / TAU_UPWELL_N_YR).exp());
    n.spread_over_ocean(geo, river, OCEAN, river_total);
    n.spread_over_ocean(geo, deep, OCEAN, upwelling);
}
