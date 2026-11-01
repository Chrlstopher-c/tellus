//! Cycle du phosphore (nutriments § 4) : altération de l'apatite (réserve finie) vers le phosphore labile des sols,
//! prélèvement, litière, minéralisation, lessivage vers l'océan, pompe biologique et remontée profonde.

use super::forcing::{Forcing, Scratch};
use super::params::{
    APATITE_K_PER_YR, APATITE_RESERVE_GTP, CP_OCEAN, CP_VEG, EXPORT_RATIO, INIT_DEEP_P_GTP, INIT_DIP_G_M2,
    LEACH_P_PER_YR, P_BURIAL_K_PER_YR, TAU_UPWELL_P_YR,
};
use super::state::{gt_from_g_m2, Element, Geometry, Layout};
use crate::shared::units::GRAMS_PER_GT;

pub(crate) const VEG: usize = 0;
pub(crate) const SOIL: usize = 1;
pub(crate) const LAB: usize = 2;
pub(crate) const OCEAN: usize = 3;
const KINDS: usize = 4;
pub(crate) const APATITE: usize = 0;
pub(crate) const RIVER: usize = 1;
pub(crate) const DEEP: usize = 2;
pub(crate) const SED: usize = 3;
const GLOBALS: usize = 4;

pub(crate) fn initial(geo: &Geometry) -> Element {
    let layout = Layout {
        cells: geo.cell_count(),
        kinds: KINDS,
        globals: GLOBALS,
    };
    let mut stocks = vec![0.0; layout.len()];
    stocks[layout.glob(APATITE)] = APATITE_RESERVE_GTP - INIT_DEEP_P_GTP;
    stocks[layout.glob(DEEP)] = INIT_DEEP_P_GTP;
    for (i, &area) in geo.areas_m2.iter().enumerate() {
        if geo.is_ocean[i] {
            stocks[layout.at(OCEAN, i)] = gt_from_g_m2(INIT_DIP_G_M2, area);
        }
    }
    Element::from_stocks(layout, stocks)
}

fn land_cell(p: &mut Element, area_m2: f64, i: usize, forcing: &Forcing, scratch: &Scratch, dt: f64) {
    let lay = p.layout;
    let (lab, veg, soil) = (lay.at(LAB, i), lay.at(VEG, i), lay.at(SOIL, i));
    let river = lay.glob(RIVER);
    let demand = forcing.npp_eff[i] * area_m2 / GRAMS_PER_GT * dt / CP_VEG;
    let mineralised = scratch.mineral_fraction[i] * p.get(soil);
    p.flow(soil, lab, mineralised);
    p.flow(lab, veg, demand);
    p.flow(veg, soil, scratch.litter_gtc[i] / CP_VEG);
    let leached = (1.0 - (-LEACH_P_PER_YR * scratch.runoff_fraction[i] * dt).exp()) * p.get(lab);
    p.flow(lab, river, leached);
}

/// Sous-pas terrestre du phosphore : prélèvement, litière, minéralisation, lessivage.
pub(crate) fn land_substep(p: &mut Element, geo: &Geometry, forcing: &Forcing, scratch: &Scratch, dt: f64) {
    for i in geo.land_cells() {
        land_cell(p, geo.areas_m2[i], i, forcing, scratch, dt);
    }
}

/// Export de phosphore organique marin vers le fond, au rapport Redfield (C:P ≈ 41 en masse).
pub(crate) fn marine_export(p: &mut Element, geo: &Geometry, forcing: &Forcing, dt: f64) {
    let deep = p.layout.glob(DEEP);
    for i in geo.ocean_cells() {
        let export = EXPORT_RATIO * forcing.npp_eff[i] * geo.areas_m2[i] / GRAMS_PER_GT * dt / CP_OCEAN;
        let source = p.layout.at(OCEAN, i);
        p.flow(source, deep, export);
    }
}

/// Altération de l'apatite (proportionnelle au climat) vers le phosphore labile des sols, en début de tick : c'est
/// l'apport annuel qui alimente le prélèvement (flux, pas stock).
pub(crate) fn weather(p: &mut Element, geo: &Geometry, weathering: f64, dt: f64) {
    let apatite = p.layout.glob(APATITE);
    let weathered = p.get(apatite) * (1.0 - (-APATITE_K_PER_YR * weathering * dt).exp());
    p.spread_over_land(geo, apatite, LAB, weathered);
}

/// Transport de fin de tick : rivière, remontée profonde, enfouissement du fond vers les sédiments.
pub(crate) fn transport(p: &mut Element, geo: &Geometry, dt: f64) {
    let (river, deep) = (p.layout.glob(RIVER), p.layout.glob(DEEP));
    let river_total = p.get(river);
    let upwelling = p.get(deep) * (1.0 - (-dt / TAU_UPWELL_P_YR).exp());
    p.spread_over_ocean(geo, river, OCEAN, river_total);
    p.spread_over_ocean(geo, deep, OCEAN, upwelling);
    let sed = p.layout.glob(SED);
    let buried = p.get(deep) * (1.0 - (-P_BURIAL_K_PER_YR * dt).exp());
    p.flow(deep, sed, buried);
}
