//! Cycle de l'eau par cellule (REGLES § 6, nutriments § 1) : précipitations, infiltration, ruissellement vers
//! l'océan, évapotranspiration. La glace transite en équilibre (stock glaciaire figé au MVP). L'évaporation océanique
//! est le résidu du bilan de vapeur : la vapeur ne fuit jamais.

use super::forcing::Scratch;
use super::inputs::MatterInputs;
use super::params::{OCEAN_WATER_KM3, PET_MM_PER_K_YR, PET_OFFSET_C, SOIL_CAP_MM, VAPOUR_KM3};
use super::state::{Element, Geometry, Layout};
use crate::shared::units::KM3_PER_MM_M2;

pub(crate) const SOIL: usize = 0;
pub(crate) const VAPOR: usize = 0;
pub(crate) const OCEAN: usize = 1;
const KINDS: usize = 1;
const GLOBALS: usize = 2;

/// Bilan hydrique d'une cellule sur un sous-pas, km³.
#[derive(Clone, Copy)]
struct CellWater {
    precip: f64,
    infil: f64,
    runoff: f64,
    et: f64,
    runoff_fraction: f64,
    wetness: f64,
}

/// Facteur d'humidité du sol ∈ [0,5 ; 1] : moitié de la capacité au champ ou plus = pleine activité.
pub(crate) fn soil_capacity_km3(area_m2: f64) -> f64 {
    SOIL_CAP_MM * area_m2 * KM3_PER_MM_M2
}

pub(crate) fn moisture_factor(stored: f64, capacity: f64) -> f64 {
    if capacity > 0.0 {
        0.5 + 0.5 * (2.0 * stored / capacity).min(1.0)
    } else {
        0.5
    }
}

fn cell_water(water: &Element, geo: &Geometry, inputs: &MatterInputs<'_>, i: usize, dt: f64) -> CellWater {
    let area = geo.areas_m2[i];
    let precip = f64::from(inputs.precip_mm_yr[i]) * area * KM3_PER_MM_M2 * dt;
    if geo.is_ocean[i] {
        return CellWater {
            precip,
            infil: 0.0,
            runoff: 0.0,
            et: 0.0,
            runoff_fraction: 0.0,
            wetness: 0.0,
        };
    }
    let ice = f64::from(inputs.ice_fraction[i]);
    let stored = water.get(water.layout.at(SOIL, i));
    let capacity = SOIL_CAP_MM * area * KM3_PER_MM_M2;
    let infil = (precip * (1.0 - ice)).min((capacity - stored).max(0.0));
    let runoff = precip - infil;
    let runoff_fraction = if precip > 0.0 { runoff / precip } else { 0.0 };
    let temperature = f64::from(inputs.temperature_c[i]);
    let demand = PET_MM_PER_K_YR * (temperature + PET_OFFSET_C).max(0.0) * area * KM3_PER_MM_M2 * dt;
    let et = demand.min(stored) * (1.0 - ice);
    let wetness = moisture_factor(stored, capacity);
    CellWater {
        precip,
        infil,
        runoff,
        et,
        runoff_fraction,
        wetness,
    }
}

/// Un sous-pas hydrique : bilan de vapeur fermé, transferts par le registre.
pub(crate) fn substep(water: &mut Element, geo: &Geometry, inputs: &MatterInputs<'_>, dt: f64, scratch: &mut Scratch) {
    let (vapor, ocean) = (water.layout.glob(VAPOR), water.layout.glob(OCEAN));
    let (mut precip_total, mut et_total) = (0.0, 0.0);
    for (i, _) in geo.areas_m2.iter().enumerate() {
        let cell = cell_water(water, geo, inputs, i, dt);
        precip_total += cell.precip;
        et_total += cell.et;
        scratch.et_km3[i] = cell.et;
        scratch.runoff_fraction[i] = cell.runoff_fraction;
        scratch.wetness[i] = cell.wetness;
    }
    for (i, &sea) in geo.is_ocean.iter().enumerate() {
        if !sea {
            water.flow(water.layout.at(SOIL, i), vapor, scratch.et_km3[i]);
        }
    }
    water.exchange(ocean, vapor, precip_total - et_total);
    for (i, _) in geo.areas_m2.iter().enumerate() {
        let cell = cell_water(water, geo, inputs, i, dt);
        if geo.is_ocean[i] {
            water.flow(vapor, ocean, cell.precip);
        } else {
            water.flow(vapor, water.layout.at(SOIL, i), cell.infil);
            water.flow(vapor, ocean, cell.runoff);
        }
    }
}

/// Stocks initiaux : sols à mi-capacité, vapeur et océan aux valeurs de la table (nutriments § 1.1).
pub(crate) fn initial(geo: &Geometry) -> Element {
    let layout = Layout {
        cells: geo.cell_count(),
        kinds: KINDS,
        globals: GLOBALS,
    };
    let mut stocks = vec![0.0; layout.len()];
    for (i, &area) in geo.areas_m2.iter().enumerate() {
        if !geo.is_ocean[i] {
            stocks[layout.at(SOIL, i)] = 0.5 * SOIL_CAP_MM * area * KM3_PER_MM_M2;
        }
    }
    stocks[layout.glob(VAPOR)] = VAPOUR_KM3;
    stocks[layout.glob(OCEAN)] = OCEAN_WATER_KM3 - VAPOUR_KM3;
    Element::from_stocks(layout, stocks)
}
