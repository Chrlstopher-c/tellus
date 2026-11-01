//! Cycle du carbone : végétation et sol à deux pools (rapide, lent) par cellule terrestre, pompe biologique et
//! couche de mélange océanique à l'équilibre de Revelle avec l'atmosphère (nutriments § 2).
//! Réservoirs linéaires intégrés exactement : stable à tout Δt.

use super::forcing::{Forcing, Scratch};
use super::params::{
    DIC_REF_GC_M2, EXPORT_RATIO, HUMIFICATION, K_FAST_PER_YR, K_SLOW_PER_YR, K_VEG_PER_YR, MANTLE_CARBON_GTC, PPM_REF,
    REVELLE, SOLUBILITY_PER_K, TAU_MIXED_LAYER_YR, TAU_UPWELL_C_YR, T_REF_C,
};
use super::state::{Element, Geometry, Layout};
use crate::shared::units::{GRAMS_PER_GT, GTC_PER_PPM};

pub(crate) const VEG: usize = 0;
pub(crate) const FAST: usize = 1;
pub(crate) const SLOW: usize = 2;
pub(crate) const MIX: usize = 3;
const KINDS: usize = 4;
pub(crate) const ATM: usize = 0;
pub(crate) const DEEP: usize = 1;
pub(crate) const SED_CARB: usize = 2;
pub(crate) const SED_ORG: usize = 3;
pub(crate) const MANTLE: usize = 4;
const GLOBALS: usize = 5;
const BISECTION_STEPS: usize = 80;

/// Capacité de la couche de mélange : `a` tel que DIC = a · pCO₂^(1/R) (GtC), à la température donnée.
pub(crate) fn mixed_layer_coef(area_m2: f64, temperature_c: f64) -> f64 {
    let dic_ref = DIC_REF_GC_M2 * area_m2 / GRAMS_PER_GT;
    let scale = PPM_REF * (SOLUBILITY_PER_K * (temperature_c - T_REF_C)).exp();
    dic_ref * scale.powf(-1.0 / REVELLE)
}

pub(crate) fn initial(geo: &Geometry, temperature_c: &[f32], co2_ppm: f64) -> Element {
    let layout = Layout {
        cells: geo.cell_count(),
        kinds: KINDS,
        globals: GLOBALS,
    };
    let mut stocks = vec![0.0; layout.len()];
    stocks[layout.glob(ATM)] = co2_ppm * GTC_PER_PPM;
    stocks[layout.glob(MANTLE)] = MANTLE_CARBON_GTC;
    let partial = co2_ppm.powf(1.0 / REVELLE);
    for (i, &area) in geo.areas_m2.iter().enumerate() {
        if geo.is_ocean[i] {
            stocks[layout.at(MIX, i)] = mixed_layer_coef(area, f64::from(temperature_c[i])) * partial;
        }
    }
    Element::from_stocks(layout, stocks)
}

/// Réservoir linéaire intégré exactement sur `dt` : apport total `inflow` (GtC), décroissance `k` (an⁻¹).
/// Renvoie (sortie totale sur le pas, stock final).
pub(crate) fn linear_pool(pre: f64, inflow: f64, k: f64, dt: f64) -> (f64, f64) {
    if k <= 0.0 {
        return (0.0, pre + inflow);
    }
    let decay = (-k * dt).exp();
    let end = pre * decay + (inflow / dt) / k * (1.0 - decay);
    ((pre + inflow - end).max(0.0), end)
}

/// Litière de végétation (GtC) : renouvellement exact, plafond de biomasse (l'excédent part en litière).
fn vegetation_litter(c: &Element, veg: usize, npp_gtc_yr: f64, cap_gtc: f64, dt: f64) -> f64 {
    let pre = c.get(veg);
    let start = pre.min(cap_gtc.max(0.0));
    let (out, _) = linear_pool(start, npp_gtc_yr * dt, K_VEG_PER_YR, dt);
    (pre - start) + out
}

fn land_cell(c: &mut Element, area_m2: f64, i: usize, forcing: &Forcing, scratch: &mut Scratch, dt: f64) {
    let lay = c.layout;
    let (atm, veg, fast, slow) = (lay.glob(ATM), lay.at(VEG, i), lay.at(FAST, i), lay.at(SLOW, i));
    let npp = forcing.npp_eff[i] * area_m2 / GRAMS_PER_GT;
    let litter = vegetation_litter(c, veg, npp, forcing.biomass_cap_gtc[i], dt);
    c.flow(atm, veg, npp * dt);
    c.flow(veg, fast, litter);
    let rate = forcing.f_temp[i] * scratch.wetness[i];
    let (fast_pre, slow_pre) = (c.get(fast) - litter, c.get(slow));
    let (fast_out, _) = linear_pool(fast_pre, litter, K_FAST_PER_YR * rate, dt);
    let to_slow = HUMIFICATION * fast_out;
    let (slow_out, _) = linear_pool(slow_pre, to_slow, K_SLOW_PER_YR * rate, dt);
    c.flow(fast, slow, to_slow);
    c.flow(fast, atm, fast_out - to_slow);
    c.flow(slow, atm, slow_out);
    let soil = fast_pre + slow_pre;
    scratch.litter_gtc[i] = litter;
    scratch.mineral_fraction[i] = if soil > 0.0 {
        ((fast_out - to_slow + slow_out) / soil).min(1.0)
    } else {
        0.0
    };
}

/// Sous-pas terrestre : NPP fixée depuis l'atmosphère, litière, décomposition, respiration hétérotrophe.
pub(crate) fn land_substep(c: &mut Element, geo: &Geometry, forcing: &Forcing, scratch: &mut Scratch, dt: f64) {
    for i in geo.land_cells() {
        land_cell(c, geo.areas_m2[i], i, forcing, scratch, dt);
    }
}

/// Export de carbone organique marin vers le fond (pompe biologique), fraction `EXPORT_RATIO` de la NPP.
pub(crate) fn marine_export(c: &mut Element, geo: &Geometry, forcing: &Forcing, dt: f64) {
    let deep = c.layout.glob(DEEP);
    for i in geo.ocean_cells() {
        let export = EXPORT_RATIO * forcing.npp_eff[i] * geo.areas_m2[i] / GRAMS_PER_GT * dt;
        let source = c.layout.at(MIX, i);
        c.flow(source, deep, export);
    }
}

/// Fin de tick : remontée profonde vers la couche de mélange, puis équilibre de la couche avec l'atmosphère.
pub(crate) fn end_of_tick(c: &mut Element, geo: &Geometry, temperature_c: &[f32], dt: f64) {
    upwell(c, geo, dt);
    equilibrate_mixed_layer(c, geo, temperature_c, dt);
}

fn upwell(c: &mut Element, geo: &Geometry, dt: f64) {
    let deep = c.layout.glob(DEEP);
    let amount = c.get(deep) * (1.0 - (-dt / TAU_UPWELL_C_YR).exp());
    for i in geo.ocean_cells() {
        let target = c.layout.at(MIX, i);
        c.flow(deep, target, amount * geo.areas_m2[i] / geo.ocean_area_m2);
    }
}

/// Pression partielle (ppm) qui répartit `total` GtC entre atmosphère et couche de mélange ; monotone, bissection.
fn solve_pressure(total_gtc: f64, coef_sum: f64) -> f64 {
    let (mut lo, mut hi) = (0.0, (total_gtc / GTC_PER_PPM).max(0.0));
    for _ in 0..BISECTION_STEPS {
        let mid = 0.5 * (lo + hi);
        let residual = GTC_PER_PPM * mid + coef_sum * mid.powf(1.0 / REVELLE) - total_gtc;
        if residual > 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    0.5 * (lo + hi)
}

fn equilibrate_mixed_layer(c: &mut Element, geo: &Geometry, temperature_c: &[f32], dt: f64) {
    let atm = c.layout.glob(ATM);
    let coefs: Vec<(usize, f64)> = geo
        .ocean_cells()
        .map(|i| (i, mixed_layer_coef(geo.areas_m2[i], f64::from(temperature_c[i]))))
        .collect();
    let coef_sum: f64 = coefs.iter().map(|&(_, a)| a).sum();
    let mixed: f64 = coefs.iter().map(|&(i, _)| c.get(c.layout.at(MIX, i))).sum();
    let total = c.get(atm) + mixed;
    let partial = solve_pressure(total, coef_sum).powf(1.0 / REVELLE);
    let relax = 1.0 - (-dt / TAU_MIXED_LAYER_YR).exp();
    for (i, a) in coefs {
        let target = c.layout.at(MIX, i);
        let delta = relax * (a * partial - c.get(target));
        c.exchange(atm, target, delta);
    }
}
