//! Circulation zonale de surface (REGLES §5, climat.md §5) : trois cellules par hémisphère, centrées sur la zone de
//! convergence intertropicale (ZCIT). Demi-largeur de Hadley : Held–Hou, φ_H ∝ Ω^(−1/2), bornée.
//! Vitesse des alizés : forme `Ωa·sin²φ/cosφ` (Held–Hou sans friction) × friction κ calée sur la Terre [décision].
//! Les rapports des autres cellules sont des choix documentés ; aucune dynamique, c'est un diagnostic pur.

#[cfg(test)]
mod tests;

use std::f64::consts::{FRAC_PI_2, PI};

/// Exposant de Held–Hou : φ_H ∝ Ω^(−k) avec k = 1/2 (Held & Hou 1980). Les sources se contredisent sur k [M].
pub const HELD_HOU_EXPONENT: f64 = 0.5;
/// Demi-largeur de la cellule de Hadley sur Terre, degrés (bord à 30°).
pub const HADLEY_HALF_WIDTH_EARTH_DEG: f64 = 30.0;
/// Bornes de la demi-largeur de Hadley, degrés.
pub const HADLEY_HALF_WIDTH_MIN_DEG: f64 = 15.0;
pub const HADLEY_HALF_WIDTH_MAX_DEG: f64 = 45.0;
/// Bord de la cellule de Ferrel en multiple de φ_H : 60° sur Terre.
pub const FERREL_EDGE_RATIO: f64 = 2.0;
/// Bord maximal de la cellule de Ferrel, degrés : laisse une cellule polaire non nulle.
const FERREL_EDGE_MAX_DEG: f64 = 80.0;
/// Friction de surface κ : alizés de l'ordre de 6 m s⁻¹ sur Terre au bord de Hadley [décision].
pub const SURFACE_FRICTION: f64 = 0.0447;
/// Amplitude zonale relative des westerlies de Ferrel et des easterlies polaires (par rapport aux alizés).
pub const FERREL_WESTERLY_RATIO: f64 = 1.3;
pub const POLAR_EASTERLY_RATIO: f64 = 0.5;
/// Amplitude méridienne relative : alizés (0,3) puis cellules de Ferrel et polaire (0,2) [décision].
pub const TRADE_MERIDIONAL_RATIO: f64 = 0.5;
pub const MID_MERIDIONAL_RATIO: f64 = 0.4;
/// Jour sidéral de référence (Terre), heures : sert à la vitesse de rotation relative.
pub const EARTH_SIDEREAL_DAY_HOURS: f64 = 23.9345;
/// Plancher de cosφ pour éviter une division par zéro aux pôles.
const COS_FLOOR: f64 = 1e-9;

/// Vitesse de Held–Hou sans friction à la latitude `lat_rad`, m s⁻¹ : `Ωa·sin²φ/cosφ`.
pub fn held_hou_upper_wind(lat_rad: f64, omega: f64, radius_m: f64) -> f64 {
    let cos = lat_rad.cos().abs().max(COS_FLOOR);
    omega * radius_m * lat_rad.sin().powi(2) / cos
}

/// Demi-largeur de la cellule de Hadley, radians, pour une vitesse de rotation relative `omega_rel` (1 = Terre).
pub fn hadley_half_width(omega_rel: f64) -> f64 {
    let degrees = HADLEY_HALF_WIDTH_EARTH_DEG * omega_rel.max(f64::MIN_POSITIVE).powf(-HELD_HOU_EXPONENT);
    degrees
        .clamp(HADLEY_HALF_WIDTH_MIN_DEG, HADLEY_HALF_WIDTH_MAX_DEG)
        .to_radians()
}

/// Géométrie et amplitude des trois cellules d'un hémisphère-couple, centrées sur la ZCIT.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circulation {
    /// Latitude de la ZCIT (centre de la cellule de Hadley), radians.
    pub centre_rad: f64,
    /// Demi-largeur de Hadley depuis la ZCIT, radians.
    pub hadley_half_rad: f64,
    /// Distance du bord de Ferrel à la ZCIT, radians.
    pub ferrel_edge_rad: f64,
    /// Vitesse zonale des alizés au bord de Hadley-friction, m s⁻¹ : amplitude de référence des trois cellules.
    pub trade_speed: f64,
}

impl Circulation {
    /// Cellules centrées en `centre_rad` pour une planète de vitesse angulaire `omega` (rad s⁻¹).
    pub fn new(centre_rad: f64, omega: f64, radius_m: f64, omega_rel: f64) -> Self {
        let half = hadley_half_width(omega_rel);
        let ferrel_edge = (FERREL_EDGE_RATIO * half).min(FERREL_EDGE_MAX_DEG.to_radians());
        let trade = SURFACE_FRICTION * held_hou_upper_wind(half, omega, radius_m);
        Self {
            centre_rad,
            hadley_half_rad: half,
            ferrel_edge_rad: ferrel_edge,
            trade_speed: trade,
        }
    }
}

/// Vent de surface `(u, v)` en m s⁻¹ (u vers l'est, v vers le nord) à la latitude `lat_rad`.
pub fn surface_wind(circ: &Circulation, lat_rad: f64) -> (f64, f64) {
    let offset = lat_rad - circ.centre_rad;
    let distance = offset.abs();
    if distance < circ.hadley_half_rad {
        hadley_wind(circ, offset)
    } else if distance < circ.ferrel_edge_rad {
        midlatitude_wind(circ, offset)
    } else {
        polar_wind(circ, offset)
    }
}

/// Cellule de Hadley : alizés d'est (maximum à mi-largeur), vent méridien convergent vers la ZCIT.
fn hadley_wind(circ: &Circulation, offset: f64) -> (f64, f64) {
    let phase = 2.0 * PI * (offset + circ.hadley_half_rad) / (2.0 * circ.hadley_half_rad);
    let trade = circ.trade_speed;
    (-trade * phase.sin().abs(), TRADE_MERIDIONAL_RATIO * trade * phase.sin())
}

/// Cellule de Ferrel : westerlies de surface, vent méridien poleward.
fn midlatitude_wind(circ: &Circulation, offset: f64) -> (f64, f64) {
    let x = (offset.abs() - circ.hadley_half_rad) / (circ.ferrel_edge_rad - circ.hadley_half_rad);
    let shape = (PI * x).sin();
    let trade = circ.trade_speed;
    (
        FERREL_WESTERLY_RATIO * trade * shape,
        offset.signum() * MID_MERIDIONAL_RATIO * trade * shape,
    )
}

/// Cellule polaire : easterlies de surface, vent méridien équatorward.
fn polar_wind(circ: &Circulation, offset: f64) -> (f64, f64) {
    let span = FRAC_PI_2 - circ.ferrel_edge_rad;
    let x = ((offset.abs() - circ.ferrel_edge_rad) / span).clamp(0.0, 1.0);
    let shape = (PI * x).sin();
    let trade = circ.trade_speed;
    (
        -POLAR_EASTERLY_RATIO * trade * shape,
        -offset.signum() * MID_MERIDIONAL_RATIO * trade * shape,
    )
}
