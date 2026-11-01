//! Forçage évalué une fois par tick à partir de l'état de début de pas : potentiel de Miami, plafond de Liebig
//! (eau via Miami, lumière, N, P), disponibilités N et P, facteur de température, altération relative.
//! Les disponibilités terrestres sont des FLUX annuels (minéralisation + fixation + altération), pas des stocks.
//! Le calcul par cellule est parallèle (rayon) ; les sommes sont faites dans l'ordre des cellules : résultat
//! identique bit à bit quel que soit le nombre de threads.

use super::inputs::MatterInputs;
use super::nitrogen;
use super::params::{
    CN_VEG, CO2_FERT_BETA, CP_VEG, ELEVATION_REF_M, FIX_CAP_GN_M2_YR, K_FAST_PER_YR, K_N_OCEAN_G_M2, K_P_OCEAN_G_M2,
    K_SLOW_PER_YR, LIGHT_SAT_W_M2, MIAMI_GC_PER_GDM, OCEAN_NPP_MAX_GC_M2_YR, PPM_REF, Q10_SOIL, T_REF_C,
    WEATHER_TEMP_PER_K,
};
use super::phosphorus;
use super::state::{gt_from_g_m2, per_area_g_m2, MatterState};
use super::water;
use crate::shared::units::GRAMS_PER_GT;
use rayon::prelude::*;

/// Forçage par cellule (gC m⁻² an⁻¹ sauf mention) et agrégats mondiaux.
pub(crate) struct Forcing {
    pub(crate) npp_cap: Vec<f64>,
    pub(crate) npp_eff: Vec<f64>,
    pub(crate) n_avail: Vec<f64>,
    pub(crate) p_avail: Vec<f64>,
    pub(crate) nutrient_ocean: Vec<f64>,
    pub(crate) f_temp: Vec<f64>,
    pub(crate) soil_turnover: Vec<f64>,
    pub(crate) biomass_cap_gtc: Vec<f64>,
    pub(crate) weathering: f64,
    pub(crate) npp_land_gtc_yr: f64,
    pub(crate) npp_ocean_gtc_yr: f64,
}

/// Tampons de sous-pas : humidité du sol, ETR, ruissellement relatif, litière et fraction minéralisée.
pub(crate) struct Scratch {
    pub(crate) wetness: Vec<f64>,
    pub(crate) et_km3: Vec<f64>,
    pub(crate) runoff_fraction: Vec<f64>,
    pub(crate) litter_gtc: Vec<f64>,
    pub(crate) mineral_fraction: Vec<f64>,
}

impl Scratch {
    pub(crate) fn new(cells: usize) -> Self {
        Self {
            wetness: vec![0.0; cells],
            et_km3: vec![0.0; cells],
            runoff_fraction: vec![0.0; cells],
            litter_gtc: vec![0.0; cells],
            mineral_fraction: vec![0.0; cells],
        }
    }
}

/// Grandeurs globales du tick, identiques pour toutes les cellules.
struct Global {
    fertilization: f64,
    weathering: f64,
    p_weather_g_m2_yr: f64,
}

/// Sorties d'une cellule, calculées en parallèle puis assemblées dans l'ordre des indices.
#[derive(Clone, Copy)]
struct CellForcing {
    cap: f64,
    eff: f64,
    n_avail: f64,
    p_avail: f64,
    nutrient: f64,
    f_temp: f64,
    turnover: f64,
    biomass_gtc: f64,
    gtc_yr: f64,
    is_ocean: bool,
}

/// Potentiel de NPP de Miami (Lieth 1975), gC m⁻² an⁻¹ : min(T, P) en matière sèche × 0,45.
pub(crate) fn miami_npp_gc(temperature_c: f64, precip_mm_yr: f64) -> f64 {
    let by_temperature = 3_000.0 / (1.0 + (1.315 - 0.119 * temperature_c).exp());
    let by_precipitation = 3_000.0 * (1.0 - (-0.000_664 * precip_mm_yr).exp());
    by_temperature.min(by_precipitation).max(0.0) * MIAMI_GC_PER_GDM
}

/// Effet fertilisant du CO₂ (nutriments § 2.2), borné à 0,1.
fn co2_fertilization(co2_ppm: f64) -> f64 {
    (1.0 + CO2_FERT_BETA * (co2_ppm.max(1.0) / PPM_REF).ln()).max(0.1)
}

/// Facteur de Q10 de la décomposition, borné à [0, 10].
pub(crate) fn q10_factor(temperature_c: f64) -> f64 {
    Q10_SOIL.powf((temperature_c - T_REF_C) / 10.0).clamp(0.0, 10.0)
}

/// Disponibilité ∈ [0, 1] : approvisionnement annuel rapporté à la demande annuelle du potentiel.
fn availability(supply_per_yr: f64, demand_per_yr: f64) -> f64 {
    if demand_per_yr > 0.0 {
        (supply_per_yr / demand_per_yr).min(1.0)
    } else {
        1.0
    }
}

/// Fraction annuelle de carbone du sol décomposée (taux de décomposition × humidité), an⁻¹.
fn soil_turnover(state: &MatterState, i: usize, f_temp: f64) -> f64 {
    let c = state.carbon.layout;
    let (fast, slow) = (
        state.carbon.get(c.at(super::carbon::FAST, i)),
        state.carbon.get(c.at(super::carbon::SLOW, i)),
    );
    let stored = state.water.get(state.water.layout.at(water::SOIL, i));
    let moisture = water::moisture_factor(stored, water::soil_capacity_km3(state.geometry.areas_m2[i]));
    let respired = (K_FAST_PER_YR * fast + K_SLOW_PER_YR * slow) * f_temp * moisture;
    if fast + slow > 0.0 {
        respired / (fast + slow)
    } else {
        0.0
    }
}

/// Plafond terrestre (Miami × CO₂ × lumière × Liebig N/P) à partir des flux annuels d'approvisionnement.
/// Azote : minéral disponible + minéralisation du sol + fixation (plafond biome). Phosphore : labile + minéralisation
/// + altération de l'apatite. Retourne [plafond, n, p, nutriment océanique (0)].
fn land_limits(
    state: &MatterState,
    inputs: &MatterInputs<'_>,
    i: usize,
    g: &Global,
    light: f64,
    turnover: f64,
) -> [f64; 4] {
    let area = state.geometry.areas_m2[i];
    let potential = miami_npp_gc(f64::from(inputs.temperature_c[i]), f64::from(inputs.precip_mm_yr[i]));
    let n = state.nitrogen.layout;
    let p = state.phosphorus.layout;
    let n_supply = per_area_g_m2(state.nitrogen.get(n.at(nitrogen::MIN, i)), area)
        + per_area_g_m2(state.nitrogen.get(n.at(nitrogen::SOIL, i)), area) * turnover
        + FIX_CAP_GN_M2_YR;
    let p_supply = per_area_g_m2(state.phosphorus.get(p.at(phosphorus::LAB, i)), area)
        + per_area_g_m2(state.phosphorus.get(p.at(phosphorus::SOIL, i)), area) * turnover
        + g.p_weather_g_m2_yr;
    let n_avail = availability(n_supply, potential / CN_VEG);
    let p_avail = availability(p_supply, potential / CP_VEG);
    let cap = potential * g.fertilization * light * n_avail.min(p_avail);
    [cap, n_avail, p_avail, 0.0]
}

/// Plafond océanique (productivité maximale × lumière × nutriment limitant) et disponibilités.
fn ocean_limits(state: &MatterState, i: usize, light: f64) -> [f64; 4] {
    let area = state.geometry.areas_m2[i];
    let din = per_area_g_m2(state.nitrogen.get(state.nitrogen.layout.at(nitrogen::OCEAN, i)), area);
    let dip = per_area_g_m2(
        state.phosphorus.get(state.phosphorus.layout.at(phosphorus::OCEAN, i)),
        area,
    );
    let n_avail = din / (din + K_N_OCEAN_G_M2);
    let p_avail = dip / (dip + K_P_OCEAN_G_M2);
    let nutrient = n_avail.min(p_avail);
    [OCEAN_NPP_MAX_GC_M2_YR * light * nutrient, n_avail, p_avail, nutrient]
}

/// Altération relative W_eff : moyenne terrestre pondérée de (P/1000)·exp(β(T − T_ref))·(1 − glace)·relief.
fn weathering_index(state: &MatterState, inputs: &MatterInputs<'_>) -> f64 {
    let (mut numerator, mut denominator) = (0.0, 0.0);
    for i in state.geometry.land_cells() {
        let runoff_factor = f64::from(inputs.precip_mm_yr[i]) / 1_000.0;
        let thermal = (WEATHER_TEMP_PER_K * (f64::from(inputs.temperature_c[i]) - T_REF_C)).exp();
        let q = runoff_factor * thermal * (1.0 - f64::from(inputs.ice_fraction[i]));
        let relief = 1.0 + f64::from(inputs.elevation_m[i]).max(0.0) / ELEVATION_REF_M;
        numerator += state.geometry.areas_m2[i] * q * relief;
        denominator += state.geometry.areas_m2[i] * relief;
    }
    if denominator > 0.0 {
        numerator / denominator
    } else {
        0.0
    }
}

impl Global {
    fn new(state: &MatterState, inputs: &MatterInputs<'_>) -> Self {
        let weathering = weathering_index(state, inputs);
        let apatite = state.phosphorus.get(state.phosphorus.layout.glob(phosphorus::APATITE));
        let p_weather_gt_yr = super::params::APATITE_K_PER_YR * weathering * apatite;
        let p_weather_g_m2_yr = if state.geometry.land_area_m2 > 0.0 {
            p_weather_gt_yr * GRAMS_PER_GT / state.geometry.land_area_m2
        } else {
            0.0
        };
        Self {
            fertilization: co2_fertilization(state.co2_ppm()),
            weathering,
            p_weather_g_m2_yr,
        }
    }
}

fn cell_forcing(state: &MatterState, inputs: &MatterInputs<'_>, i: usize, g: &Global) -> CellForcing {
    let area = state.geometry.areas_m2[i];
    let ice = f64::from(inputs.ice_fraction[i]);
    let light = (f64::from(inputs.insolation_w_m2[i]) / LIGHT_SAT_W_M2).clamp(0.0, 1.0) * (1.0 - ice);
    let is_ocean = state.geometry.is_ocean[i];
    let f_temp = if is_ocean {
        0.0
    } else {
        q10_factor(f64::from(inputs.temperature_c[i])) * (1.0 - ice)
    };
    let turnover = if is_ocean { 0.0 } else { soil_turnover(state, i, f_temp) };
    let [cap, n_avail, p_avail, nutrient] = if is_ocean {
        ocean_limits(state, i, light)
    } else {
        land_limits(state, inputs, i, g, light, turnover)
    };
    let eff = f64::from(inputs.npp_realized_gc_m2_yr[i]).min(cap).max(0.0);
    CellForcing {
        cap,
        eff,
        n_avail,
        p_avail,
        nutrient,
        f_temp,
        turnover,
        biomass_gtc: gt_from_g_m2(f64::from(inputs.biomass_gc_m2[i]), area),
        gtc_yr: gt_from_g_m2(eff, area),
        is_ocean,
    }
}

impl Forcing {
    pub(crate) fn compute(state: &MatterState, inputs: &MatterInputs<'_>) -> Self {
        let cells = state.geometry.cell_count();
        let g = Global::new(state, inputs);
        let per_cell: Vec<CellForcing> = (0..cells)
            .into_par_iter()
            .map(|i| cell_forcing(state, inputs, i, &g))
            .collect();
        Self::assemble(&per_cell, g.weathering)
    }

    /// Rassemble les cellules dans l'ordre des indices : les sommes globales ne dépendent pas des threads.
    fn assemble(per_cell: &[CellForcing], weathering: f64) -> Self {
        let mut forcing = Self {
            npp_cap: Vec::with_capacity(per_cell.len()),
            npp_eff: Vec::with_capacity(per_cell.len()),
            n_avail: Vec::with_capacity(per_cell.len()),
            p_avail: Vec::with_capacity(per_cell.len()),
            nutrient_ocean: Vec::with_capacity(per_cell.len()),
            f_temp: Vec::with_capacity(per_cell.len()),
            soil_turnover: Vec::with_capacity(per_cell.len()),
            biomass_cap_gtc: Vec::with_capacity(per_cell.len()),
            weathering,
            npp_land_gtc_yr: 0.0,
            npp_ocean_gtc_yr: 0.0,
        };
        for c in per_cell {
            forcing.npp_cap.push(c.cap);
            forcing.npp_eff.push(c.eff);
            forcing.n_avail.push(c.n_avail);
            forcing.p_avail.push(c.p_avail);
            forcing.nutrient_ocean.push(c.nutrient);
            forcing.f_temp.push(c.f_temp);
            forcing.soil_turnover.push(c.turnover);
            forcing.biomass_cap_gtc.push(c.biomass_gtc);
            if c.is_ocean {
                forcing.npp_ocean_gtc_yr += c.gtc_yr;
            } else {
                forcing.npp_land_gtc_yr += c.gtc_yr;
            }
        }
        forcing
    }

    /// NPP réalisée mondiale, GtC an⁻¹ (terre + océan).
    pub(crate) fn npp_global_gtc(&self) -> f64 {
        self.npp_land_gtc_yr + self.npp_ocean_gtc_yr
    }
}
