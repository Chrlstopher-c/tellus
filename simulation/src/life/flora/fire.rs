//! Feu stochastique : probabilité par an selon la charge de litière et la sécheresse, puis perte de biomasse par type.

use super::pft::{LAND_PFT, PFTS};
use super::{FloraCell, FloraEnv, FloraFlux};

const IGNITION_PER_YR: f64 = 0.05;
const FUEL_SATURATION: f64 = 0.02;
const BURN_BASE: f64 = 0.9;
const COMBUSTION_SHARE: f64 = 0.5;

/// P(feu) = 1 − exp(−λ · F · (1 − H) · Δt), F = charge saturante de litière, H = humidité du sol.
pub fn fire_probability(litter_gc_m2: f64, soil_moisture: f64, span_yr: f64) -> f64 {
    let fuel = 1.0 - (-FUEL_SATURATION * litter_gc_m2.max(0.0)).exp();
    let dryness = 1.0 - soil_moisture.clamp(0.0, 1.0);
    1.0 - (-IGNITION_PER_YR * fuel * dryness * span_yr).exp()
}

/// Applique un feu si le tirage `draw` (uniforme) est sous la probabilité ; la perte dépend de la tolérance.
pub fn apply_fire(cell: &mut FloraCell, env: &FloraEnv, span_yr: f64, draw: f64, flux: &mut FloraFlux) -> bool {
    if env.is_ocean || draw >= fire_probability(cell.litter_gc_m2, env.soil_moisture, span_yr) {
        return false;
    }
    for (i, pft) in PFTS.iter().enumerate().take(LAND_PFT) {
        let fraction = BURN_BASE * (1.0 - pft.fire_tolerance);
        let burned = cell.biomass_gc_m2[i] * fraction;
        cell.biomass_gc_m2[i] -= burned;
        cell.cover[i] *= 1.0 - fraction;
        cell.litter_gc_m2 += burned * (1.0 - COMBUSTION_SHARE);
        flux.combusted_gc_m2 += burned * COMBUSTION_SHARE;
        flux.litter_in_gc_m2 += burned * (1.0 - COMBUSTION_SHARE);
    }
    flux.burned = true;
    true
}
