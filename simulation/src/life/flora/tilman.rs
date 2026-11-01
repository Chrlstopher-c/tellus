//! Couverture par compétition colonisation-extinction de Tilman, et NPP limitée (Miami, azote, phosphore).

use super::pft::{LAND_PFT, PFTS, PFT_COUNT};

/// Plancher de colonisation par les graines voisines (immigration), fraction de couverture.
const SEED_FLOOR: f64 = 1e-4;
/// Mortalité supplémentaire hors de sa niche climatique.
const CLIMATE_STRESS: f64 = 0.2;
const MAX_STEP_YR: f64 = 0.1;
const MAX_SUBSTEPS: u32 = 200;
/// Rapport gC/gN et gC/gP des tissus végétaux.
pub const CN_PLANT: f64 = 40.0;
pub const CP_PLANT: f64 = 500.0;

/// Modèle de Miami (Lieth 1975) : min(NPP_T, NPP_P), gC m⁻² an⁻¹.
pub fn miami_npp(temp_c: f64, precip_mm: f64) -> f64 {
    let by_temp = 3000.0 / (1.0 + (1.315 - 0.119 * temp_c).exp());
    let by_precip = 3000.0 * (1.0 - (-0.000_664 * precip_mm.max(0.0)).exp());
    by_temp.min(by_precip)
}

/// Facteur de fertilisation par le CO₂ (racine du rapport à 280 ppm, borné).
pub fn co2_factor(co2_ppm: f64) -> f64 {
    (co2_ppm.max(1.0) / 280.0).sqrt().clamp(0.5, 2.0)
}

/// Tilman : dC_i = c_i · C_i · (1 − Σ_{j≤i} C_j) − m_i · C_i, du plus compétitif au moins compétitif.
pub fn update_cover(cover: &mut [f64; PFT_COUNT], suit: &[f64; PFT_COUNT], neighbour: &[f64; PFT_COUNT], span: f64) {
    let steps = ((span / MAX_STEP_YR).ceil() as u32).clamp(1, MAX_SUBSTEPS);
    let h = span / f64::from(steps);
    for _ in 0..steps {
        colonise_once(cover, suit, neighbour, h);
    }
    normalise(cover);
}

fn colonise_once(cover: &mut [f64; PFT_COUNT], suit: &[f64; PFT_COUNT], neighbour: &[f64; PFT_COUNT], h: f64) {
    for i in 0..LAND_PFT {
        let pft = &PFTS[i];
        let s = suit[i];
        let above: f64 = cover[..=i].iter().sum();
        let free = (1.0 - above).max(0.0);
        let seed = neighbour[i] + SEED_FLOOR;
        let growth = pft.colonisation * s * (cover[i] + seed) * free;
        let loss = (pft.mortality + CLIMATE_STRESS * (1.0 - s)) * cover[i];
        cover[i] = (cover[i] + (growth - loss) * h).clamp(0.0, 1.0);
    }
}

/// Somme des couvertures terrestres bornée à 1 (mise à l'échelle proportionnelle).
fn normalise(cover: &mut [f64; PFT_COUNT]) {
    let total: f64 = cover[..LAND_PFT].iter().sum();
    if total > 1.0 {
        for c in cover[..LAND_PFT].iter_mut() {
            *c /= total;
        }
    }
}

/// NPP potentielle par type, avant limitation : couverture × NPP max × adéquation × CO₂.
pub fn potential_npp(cover: &[f64; PFT_COUNT], suit: &[f64; PFT_COUNT], co2_factor: f64) -> [f64; PFT_COUNT] {
    let mut pot = [0.0; PFT_COUNT];
    for (i, value) in pot.iter_mut().enumerate() {
        *value = cover[i] * PFTS[i].npp_max_gc_m2_yr * suit[i] * co2_factor;
    }
    pot
}

/// Limitation de la NPP par le minimum de Liebig : climat (Miami), plafond de la matière, azote, phosphore.
pub fn limit_npp(pot: &[f64; PFT_COUNT], limit_gc_m2_yr: f64) -> [f64; PFT_COUNT] {
    let total: f64 = pot.iter().sum();
    if total <= 0.0 {
        return [0.0; PFT_COUNT];
    }
    let lambda = (limit_gc_m2_yr / total).clamp(0.0, 1.0);
    let mut out = [0.0; PFT_COUNT];
    for (o, p) in out.iter_mut().zip(pot.iter()) {
        *o = p * lambda;
    }
    out
}
