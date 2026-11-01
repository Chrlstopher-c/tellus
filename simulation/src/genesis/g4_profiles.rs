//! G4 : profils de frontière. Chaque cellule reçoit le profil de la frontière la plus proche :
//! h(d) = A·exp(−d² / 2σ²), d = distance de croissance à la frontière. Les valeurs sont celles de `genese.md` §2.4.

use super::g2_motion::Plate;
use super::g3_boundaries::{BoundaryCell, BoundaryKind};
use super::graph::{arc_m, grow, Field, UNREACHED};
use super::GenesisError;
use crate::shared::grid::Grid;

/// Amplitude (m, négative = dépression) et échelle de décroissance (m) d'un profil.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Profile {
    pub(super) amplitude_m: f64,
    pub(super) sigma_m: f64,
}

const TRENCH: Profile = Profile {
    amplitude_m: -7_000.0,
    sigma_m: 50_000.0,
};
const ARC: Profile = Profile {
    amplitude_m: 1_500.0,
    sigma_m: 100_000.0,
};
const OROGEN: Profile = Profile {
    amplitude_m: 3_000.0,
    sigma_m: 150_000.0,
};
const COLLISION: Profile = Profile {
    amplitude_m: 4_000.0,
    sigma_m: 300_000.0,
};
const RIFT: Profile = Profile {
    amplitude_m: -750.0,
    sigma_m: 100_000.0,
};
const NO_PROFILE: Profile = Profile {
    amplitude_m: 0.0,
    sigma_m: 1.0,
};

/// Profil d'une frontière vu d'une cellule : type de frontière, nature continentale de la cellule et de
/// l'autre plaque, et si la cellule appartient à la plaque qui plonge.
pub(super) fn profile_for(
    kind: BoundaryKind,
    own_continental: bool,
    other_continental: bool,
    subducting: bool,
) -> Profile {
    match (kind, own_continental, other_continental) {
        (BoundaryKind::Convergent, true, true) => COLLISION,
        (BoundaryKind::Convergent, true, false) => OROGEN,
        (BoundaryKind::Convergent, false, true) => TRENCH,
        (BoundaryKind::Convergent, false, false) if subducting => TRENCH,
        (BoundaryKind::Convergent, false, false) => ARC,
        (BoundaryKind::Divergent, true, _) => RIFT,
        _ => NO_PROFILE,
    }
}

/// Champ tectonique (m) : profil gaussien de la frontière la plus proche de chaque cellule.
pub(super) fn tectonic_field(
    grid: &Grid,
    plate_of: &[u32],
    plates: &[Plate],
    boundaries: &[Option<BoundaryCell>],
) -> Result<Vec<f64>, GenesisError> {
    let sources: Vec<usize> = (0..grid.cell_count()).filter(|&c| boundaries[c].is_some()).collect();
    if sources.is_empty() {
        return Err(GenesisError::NoBoundary);
    }
    let field = grow(grid, &sources, |from, to| Some(arc_m(grid, from, to)));
    Ok((0..grid.cell_count())
        .map(|cell| cell_value(cell, &field, (&sources, boundaries), (plate_of, plates)))
        .collect())
}

/// Valeur du profil en `cell` : profil de la frontière d'origine, à la distance de croissance.
fn cell_value(
    cell: usize,
    field: &Field,
    (sources, boundaries): (&[usize], &[Option<BoundaryCell>]),
    (plate_of, plates): (&[u32], &[Plate]),
) -> f64 {
    let origin = field.origin[cell];
    if origin == UNREACHED {
        return 0.0;
    }
    let Some(boundary) = boundaries[sources[origin as usize]] else {
        return 0.0;
    };
    let own = plate_of[cell];
    let other = if boundary.own == own {
        boundary.other
    } else {
        boundary.own
    };
    let profile = profile_for(
        boundary.kind,
        plates[own as usize].continental,
        plates[other as usize].continental,
        boundary.subducting == own,
    );
    let d = field.dist[cell] / profile.sigma_m;
    profile.amplitude_m * (-0.5 * d * d).exp()
}
