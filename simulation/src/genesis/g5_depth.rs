//! G5 : âge océanique, profondeur GDH1 (Stein & Stein 1992) et isostasie d'Airy pour les continents.

use super::g2_motion::Plate;
use super::g3_boundaries::{BoundaryCell, BoundaryKind};
use super::graph::{arc_m, grow, UNREACHED};
use crate::shared::grid::Grid;
use crate::shared::units::{
    CRUST_REFERENCE_THICKNESS_M, GDH1_DECAY_PER_MA, GDH1_DEEP_AMP_M, GDH1_DEEP_DEPTH_M, GDH1_RIDGE_DEPTH_M,
    GDH1_SQRT_COEFF_M, GDH1_TRANSITION_MA, RHO_CONTINENTAL_CRUST, RHO_MANTLE, SECONDS_PER_MA,
};

/// Âge maximal d'une croûte océanique (Ma), atteint loin de toute dorsale.
pub(super) const AGE_CAP_MA: f64 = 180.0;

/// Profondeur d'un océan d'âge `age_ma` (m, positive vers le bas), GDH1.
pub fn gdh1_depth_m(age_ma: f64) -> f64 {
    if age_ma <= GDH1_TRANSITION_MA {
        GDH1_RIDGE_DEPTH_M + GDH1_SQRT_COEFF_M * age_ma.sqrt()
    } else {
        GDH1_DEEP_DEPTH_M - GDH1_DEEP_AMP_M * (-GDH1_DECAY_PER_MA * age_ma).exp()
    }
}

/// Élévation d'équilibre (m) d'une colonne continentale d'épaisseur `thickness_m` (Airy, niveau zéro à 35 km).
pub fn airy_elevation_m(thickness_m: f64) -> f64 {
    (thickness_m - CRUST_REFERENCE_THICKNESS_M) * (RHO_MANTLE - RHO_CONTINENTAL_CRUST) / RHO_MANTLE
}

/// Dorsale : divergence dont la cellule est océanique (océan–océan, ou bord océanique d'un rift qui s'ouvre).
pub(super) fn is_ridge(boundary: &BoundaryCell, plates: &[Plate]) -> bool {
    boundary.kind == BoundaryKind::Divergent && !plates[boundary.own as usize].continental
}

/// Âge océanique (Ma) : distance à la dorsale la plus proche (le long de la croûte océanique) divisée par la
/// demi-vitesse d'écartement. Zéro sur les plaques continentales.
pub(super) fn ocean_ages(
    grid: &Grid,
    plate_of: &[u32],
    plates: &[Plate],
    boundaries: &[Option<BoundaryCell>],
) -> Vec<f64> {
    let ridges: Vec<usize> = (0..grid.cell_count())
        .filter(|&c| boundaries[c].is_some_and(|b| is_ridge(&b, plates)))
        .collect();
    let half_speed: Vec<f64> = ridges
        .iter()
        .map(|&c| boundaries[c].map_or(0.0, |b| -0.5 * b.approach_m_s))
        .collect();
    let field = grow(grid, &ridges, |from, to| {
        let oceanic = !plates[plate_of[to] as usize].continental;
        oceanic.then(|| arc_m(grid, from, to))
    });
    (0..grid.cell_count())
        .map(|cell| {
            let origin = field.origin[cell];
            if plates[plate_of[cell] as usize].continental {
                return 0.0;
            }
            if origin == UNREACHED || half_speed[origin as usize] <= 0.0 {
                return AGE_CAP_MA;
            }
            (field.dist[cell] / half_speed[origin as usize] / SECONDS_PER_MA).min(AGE_CAP_MA)
        })
        .collect()
}

/// Hauteur de base (m, avant relief) : océan = −GDH1(âge) + profil tectonique ; continent = Airy de la
/// racine isostatique équivalente au profil (épaisseur = 35 km + profil·ρm/(ρm − ρc)).
pub(super) fn base_heights(plate_of: &[u32], plates: &[Plate], ages_ma: &[f64], tectonic: &[f64]) -> Vec<f64> {
    let root_ratio = RHO_MANTLE / (RHO_MANTLE - RHO_CONTINENTAL_CRUST);
    (0..plate_of.len())
        .map(|cell| {
            if plates[plate_of[cell] as usize].continental {
                airy_elevation_m(CRUST_REFERENCE_THICKNESS_M + tectonic[cell] * root_ratio)
            } else {
                -gdh1_depth_m(ages_ma[cell]) + tectonic[cell]
            }
        })
        .collect()
}
