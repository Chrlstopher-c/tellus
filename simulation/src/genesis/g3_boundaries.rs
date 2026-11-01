//! G3 : classification des frontières. Pour une arête (a, b), n̂ est la normale tangente de a vers b et
//! Δv = v_a − v_b au milieu de l'arête. Approche = Δv·n̂ (> 0 : convergence) ; cisaillement = composante orthogonale.

use super::g2_motion::{surface_velocity, Plate};
use super::vec3::{add, dot, norm, normalize, scale, sub, Vec3};
use crate::shared::grid::Grid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BoundaryKind {
    Convergent,
    Divergent,
    Transform,
}

/// Frontière vue depuis une cellule : type, vitesse d'approche (m/s, > 0 = convergence), plaques en présence
/// et plaque qui plonge (sous-duction, utile pour les fosses océan–océan).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct BoundaryCell {
    pub(super) kind: BoundaryKind,
    pub(super) approach_m_s: f64,
    pub(super) own: u32,
    pub(super) other: u32,
    pub(super) subducting: u32,
}

/// Classe une frontière à partir de Δv et de la normale tangente n̂. Renvoie le type et l'approche Δv·n̂.
pub(super) fn classify_motion(dv: Vec3, n: Vec3) -> (BoundaryKind, f64) {
    let approach = dot(dv, n);
    let shear = norm(sub(dv, scale(n, approach)));
    let kind = if approach > shear {
        BoundaryKind::Convergent
    } else if -approach > shear {
        BoundaryKind::Divergent
    } else {
        BoundaryKind::Transform
    };
    (kind, approach)
}

struct Edge {
    kind: BoundaryKind,
    approach_m_s: f64,
    subducting: u32,
}

/// Arête (a, b) entre les plaques `ids` = (plaque de a, plaque de b). Symétrique : la plaque qui plonge
/// est la même vue de a ou de b (la plus rapide vers l'autre, à égalité la plaque d'indice le plus bas).
fn edge(grid: &Grid, cells: (usize, usize), ids: (u32, u32), plates: &[Plate]) -> Edge {
    let centers = grid.centers();
    let (xa, xb) = (centers[cells.0], centers[cells.1]);
    let mid = normalize(add(xa, xb));
    let chord = sub(xb, xa);
    let n = normalize(sub(chord, scale(mid, dot(chord, mid))));
    let radius = grid.radius_m();
    let va = surface_velocity(&plates[ids.0 as usize], mid, radius);
    let vb = surface_velocity(&plates[ids.1 as usize], mid, radius);
    let (kind, approach_m_s) = classify_motion(sub(va, vb), n);
    let toward_a = dot(va, n);
    let toward_b = -dot(vb, n);
    let a_subducts = toward_a > toward_b || (toward_a == toward_b && ids.0 < ids.1);
    Edge {
        kind,
        approach_m_s,
        subducting: if a_subducts { ids.0 } else { ids.1 },
    }
}

/// Frontière de la cellule : arête la plus forte (|approche|) vers une plaque voisine différente.
fn boundary_at(grid: &Grid, cell: usize, plate_of: &[u32], plates: &[Plate]) -> Option<BoundaryCell> {
    let own = plate_of[cell];
    let mut best: Option<(Edge, u32)> = None;
    for &nb in grid.neighbors(cell) {
        let other = plate_of[nb as usize];
        if other == own {
            continue;
        }
        let candidate = edge(grid, (cell, nb as usize), (own, other), plates);
        if best
            .as_ref()
            .is_none_or(|(e, _)| candidate.approach_m_s.abs() > e.approach_m_s.abs())
        {
            best = Some((candidate, other));
        }
    }
    best.map(|(e, other)| BoundaryCell {
        kind: e.kind,
        approach_m_s: e.approach_m_s,
        own,
        other,
        subducting: e.subducting,
    })
}

/// Frontière de chaque cellule (`None` à l'intérieur d'une plaque).
pub(super) fn classify_cells(grid: &Grid, plate_of: &[u32], plates: &[Plate]) -> Vec<Option<BoundaryCell>> {
    (0..grid.cell_count())
        .map(|cell| boundary_at(grid, cell, plate_of, plates))
        .collect()
}
