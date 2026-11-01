//! Flore : 5 types terrestres en compétition (Tilman), plancton marin, NPP limitée, feu, litière.

pub mod biome;
pub mod fire;
pub mod pft;
pub mod tilman;

pub use pft::{suitability, Pft, LAND_PFT, PFTS, PFT_COUNT, PLANKTON};

use fire::apply_fire;
use tilman::{co2_factor, limit_npp, miami_npp, potential_npp, update_cover, CN_PLANT, CP_PLANT};

/// État végétal d'une cellule : couvertures, biomasses (gC m⁻²) et litière.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FloraCell {
    pub cover: [f64; PFT_COUNT],
    pub biomass_gc_m2: [f64; PFT_COUNT],
    pub litter_gc_m2: f64,
    pub npp_gc_m2_yr: f64,
}

/// Forçages d'une cellule pour un pas de flore. `fire_u` est le tirage uniforme du feu.
#[derive(Debug, Clone, PartialEq)]
pub struct FloraEnv {
    pub temp_c: f64,
    pub precip_mm: f64,
    pub soil_moisture: f64,
    pub npp_cap: f64,
    pub n_avail: f64,
    pub p_avail: f64,
    pub co2_ppm: f64,
    pub is_ocean: bool,
    pub neighbor_cover: [f64; PFT_COUNT],
    pub fire_u: f64,
}

/// Flux de carbone d'une cellule sur un pas, gC m⁻².
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FloraFlux {
    pub fixed_gc_m2: f64,
    pub litter_in_gc_m2: f64,
    pub decomposed_gc_m2: f64,
    pub combusted_gc_m2: f64,
    pub burned: bool,
}

/// Fait avancer la flore d'une cellule de `span` ans. Renvoie les flux de carbone correspondants.
pub fn step_cell(cell: &mut FloraCell, env: &FloraEnv, span: f64) -> FloraFlux {
    let suit = suitabilities(env);
    if env.is_ocean {
        cell.cover = [0.0; PFT_COUNT];
        cell.cover[PLANKTON] = 1.0;
    } else {
        update_cover(&mut cell.cover, &suit, &env.neighbor_cover, span);
    }
    let pot = potential_npp(&cell.cover, &suit, co2_factor(env.co2_ppm));
    let limit = env
        .npp_cap
        .min(miami_npp(env.temp_c, env.precip_mm))
        .min(env.n_avail * CN_PLANT)
        .min(env.p_avail * CP_PLANT);
    let npp = limit_npp(&pot, limit.max(0.0));
    let mut flux = FloraFlux::default();
    grow(cell, &npp, span, &mut flux);
    apply_fire(cell, env, span, env.fire_u, &mut flux);
    flux.decomposed_gc_m2 = decompose(cell, env.temp_c, span);
    cell.npp_gc_m2_yr = npp.iter().sum();
    flux
}

fn suitabilities(env: &FloraEnv) -> [f64; PFT_COUNT] {
    let mut suit = [0.0; PFT_COUNT];
    for (i, value) in suit.iter_mut().enumerate() {
        *value = suitability(&PFTS[i], env.temp_c, env.precip_mm);
    }
    suit
}

/// Biomasse relaxée vers NPP × séjour (solution exacte sur le pas) ; la différence est la litière produite.
fn grow(cell: &mut FloraCell, npp: &[f64; PFT_COUNT], span: f64, flux: &mut FloraFlux) {
    for (i, pft) in PFTS.iter().enumerate() {
        let decay = (-span / pft.residence_yr).exp();
        let before = cell.biomass_gc_m2[i];
        let after = before * decay + npp[i] * pft.residence_yr * (1.0 - decay);
        let turnover = (before + npp[i] * span - after).max(0.0);
        cell.biomass_gc_m2[i] = after;
        cell.litter_gc_m2 += turnover;
        flux.litter_in_gc_m2 += turnover;
        flux.fixed_gc_m2 += npp[i] * span;
    }
}

/// Décomposition de la litière, Q10 = 2 autour de 10 °C ; renvoie le carbone rendu à l'atmosphère.
fn decompose(cell: &mut FloraCell, temp_c: f64, span: f64) -> f64 {
    let rate = 0.5 * 2f64.powf((temp_c.clamp(-20.0, 40.0) - 10.0) / 10.0);
    let out = cell.litter_gc_m2 * (1.0 - (-rate * span).exp());
    cell.litter_gc_m2 -= out;
    out
}
