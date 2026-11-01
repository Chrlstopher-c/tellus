//! Maillage de calcul du climat : aires unitaires, poids d'arêtes duales et Laplacien sphérique (volumes finis).
//! Toutes les longueurs sont sur la sphère unitaire (radians, stéradians) : le rayon n'intervient qu'à l'affichage.

#[cfg(test)]
mod tests;

use crate::shared::grid::Grid;
use std::fmt;

type Vec3 = [f64; 3];

/// Erreur de topologie : une arête du graphe n'est pas bordée par deux triangles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeshError {
    OpenEdge { cell: usize, neighbor: usize },
}

impl fmt::Display for MeshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenEdge { cell, neighbor } => {
                write!(f, "arête ({cell},{neighbor}) non bordée par deux triangles")
            }
        }
    }
}

impl std::error::Error for MeshError {}

/// Géométrie figée de la grille pour le climat. `weights[k]` = ℓ/δ de l'arête `k` (longueur duale sur distance).
pub struct Mesh {
    area_sr: Vec<f64>,
    edge_offsets: Vec<u32>,
    neighbor_indices: Vec<u32>,
    edge_weights: Vec<f64>,
    edge_flow: Vec<Vec3>,
    east: Vec<Vec3>,
    north: Vec<Vec3>,
    sin_lat: Vec<f64>,
    radius_m: f64,
}

impl Mesh {
    pub fn new(grid: &Grid) -> Result<Self, MeshError> {
        let centers = grid.centers();
        let triangles = grid.triangles();
        let circum = circumcenters(triangles, centers);
        let incident = incident_triangles(triangles, centers.len());
        let edge_offsets = grid.neighbor_offsets().to_vec();
        let neighbor_indices = grid.neighbor_indices().to_vec();
        let ctx = EdgeContext {
            triangles,
            centers,
            circum: &circum,
            incident: &incident,
        };
        let edge_weights = edge_weights(&edge_offsets, &neighbor_indices, &ctx)?;
        let edge_flow = edge_flows(&edge_offsets, &neighbor_indices, &edge_weights, centers);
        let (east, north) = local_frames(centers);
        let radius = grid.radius_m();
        let area_sr = grid.areas_m2().iter().map(|a| a / (radius * radius)).collect();
        let sin_lat = centers.iter().map(|c| c[1]).collect();
        Ok(Self {
            area_sr,
            edge_offsets,
            neighbor_indices,
            edge_weights,
            edge_flow,
            east,
            north,
            sin_lat,
            radius_m: radius,
        })
    }

    pub fn cell_count(&self) -> usize {
        self.area_sr.len()
    }

    /// Rayon de la sphère, m.
    pub fn radius_m(&self) -> f64 {
        self.radius_m
    }

    /// Flux géométrique `ℓ·n` par arête orientée (radians), `n` tangent sortant de la cellule source.
    /// Antisymétrique exactement : `edge_flow` de (i→j) = −`edge_flow` de (j→i).
    pub fn edge_flow(&self) -> &[Vec3] {
        &self.edge_flow
    }

    /// Vecteur unitaire vers l'est de chaque cellule (nul au pôle exact).
    pub fn east(&self) -> &[Vec3] {
        &self.east
    }

    /// Vecteur unitaire vers le nord de chaque cellule (nul au pôle exact).
    pub fn north(&self) -> &[Vec3] {
        &self.north
    }

    /// Aire de chaque cellule, stéradians (somme 4π).
    pub fn area_sr(&self) -> &[f64] {
        &self.area_sr
    }

    /// Sinus de la latitude de la cellule (axe y, convention de `insolation` et de la table texel).
    pub fn sin_lat(&self, cell: usize) -> f64 {
        self.sin_lat[cell]
    }

    /// Sinus de la latitude de toutes les cellules.
    pub fn sin_lats(&self) -> &[f64] {
        &self.sin_lat
    }

    /// Voisins et poids d'arête de la cellule.
    pub fn edges(&self, cell: usize) -> impl Iterator<Item = (usize, f64)> + '_ {
        let range = self.edge_range(cell);
        range.map(move |k| (self.neighbor_indices[k] as usize, self.edge_weights[k]))
    }

    /// Décalages CSR des arêtes (longueur `cell_count + 1`).
    pub fn edge_offsets(&self) -> &[u32] {
        &self.edge_offsets
    }

    pub fn neighbor_indices(&self) -> &[u32] {
        &self.neighbor_indices
    }

    pub fn edge_weights(&self) -> &[f64] {
        &self.edge_weights
    }

    /// Laplacien sphérique unitaire ∇²T (rad⁻²) : `out[i] = (1/a_i) Σ_j w_ij (T_j − T_i)`.
    pub fn laplacian(&self, field: &[f64], out: &mut [f64]) {
        for (cell, slot) in out.iter_mut().enumerate() {
            let flux: f64 = self.edges(cell).map(|(j, w)| w * (field[j] - field[cell])).sum();
            *slot = flux / self.area_sr[cell];
        }
    }

    fn edge_range(&self, cell: usize) -> std::ops::Range<usize> {
        self.edge_offsets[cell] as usize..self.edge_offsets[cell + 1] as usize
    }
}

fn circumcenters(triangles: &[[u32; 3]], centers: &[Vec3]) -> Vec<Vec3> {
    triangles
        .iter()
        .map(|tri| {
            let [a, b, c] = tri.map(|v| centers[v as usize]);
            circumcenter(a, b, c)
        })
        .collect()
}

/// Circoncentre sphérique : équidistant des trois sommets, du côté du triangle.
fn circumcenter(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let n = cross(sub(b, a), sub(c, a));
    let sign = if dot(n, a) < 0.0 { -1.0 } else { 1.0 };
    normalize(scale(n, sign))
}

fn incident_triangles(triangles: &[[u32; 3]], vertex_count: usize) -> Vec<Vec<u32>> {
    let mut incident = vec![Vec::with_capacity(6); vertex_count];
    for (t, tri) in triangles.iter().enumerate() {
        for &v in tri {
            incident[v as usize].push(t as u32);
        }
    }
    incident
}

struct EdgeContext<'a> {
    triangles: &'a [[u32; 3]],
    centers: &'a [Vec3],
    circum: &'a [Vec3],
    incident: &'a [Vec<u32>],
}

fn edge_weights(offsets: &[u32], neighbors: &[u32], ctx: &EdgeContext<'_>) -> Result<Vec<f64>, MeshError> {
    let mut weights = Vec::with_capacity(neighbors.len());
    for (cell, window) in offsets.windows(2).enumerate() {
        for &neighbor in &neighbors[window[0] as usize..window[1] as usize] {
            let neighbor = neighbor as usize;
            let (t1, t2) = bordering_triangles(ctx, cell, neighbor).ok_or(MeshError::OpenEdge { cell, neighbor })?;
            let dual = angle(ctx.circum[t1], ctx.circum[t2]);
            let dist = angle(ctx.centers[cell], ctx.centers[neighbor]);
            weights.push(dual / dist);
        }
    }
    Ok(weights)
}

/// Les deux triangles qui contiennent l'arête (cell, neighbor), ou `None` si l'arête n'est pas bordée deux fois.
fn bordering_triangles(ctx: &EdgeContext<'_>, cell: usize, neighbor: usize) -> Option<(usize, usize)> {
    let target = neighbor as u32;
    let found: Vec<usize> = ctx.incident[cell]
        .iter()
        .map(|&t| t as usize)
        .filter(|&t| ctx.triangles[t].contains(&target))
        .collect();
    match found.as_slice() {
        [a, b] => Some((*a, *b)),
        _ => None,
    }
}

fn angle(a: Vec3, b: Vec3) -> f64 {
    norm(cross(a, b)).atan2(dot(a, b))
}

fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: Vec3, s: f64) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn norm(a: Vec3) -> f64 {
    dot(a, a).sqrt()
}

fn normalize(a: Vec3) -> Vec3 {
    scale(a, 1.0 / norm(a))
}

fn safe_normalize(a: Vec3) -> Vec3 {
    let length = norm(a);
    if length > 1e-12 {
        scale(a, 1.0 / length)
    } else {
        [0.0; 3]
    }
}

fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Flux `ℓ·n` par arête orientée. Le calcul se fait dans l'ordre canonique (cellule basse → haute) puis le signe
/// est appliqué : l'antisymétrie est exacte, donc la vapeur transportée est conservée au bit près.
fn edge_flows(offsets: &[u32], neighbors: &[u32], weights: &[f64], centers: &[Vec3]) -> Vec<Vec3> {
    let mut flows = Vec::with_capacity(neighbors.len());
    for (cell, window) in offsets.windows(2).enumerate() {
        for k in window[0] as usize..window[1] as usize {
            let other = neighbors[k] as usize;
            let (low, high, sign) = if cell < other { (cell, other, 1.0) } else { (other, cell, -1.0) };
            let length = weights[k] * angle(centers[cell], centers[other]);
            let normal = chord_normal(centers[low], centers[high]);
            flows.push(scale(normal, sign * length));
        }
    }
    flows
}

/// Normale unitaire de l'arête duale entre `low` et `high` : la corde, projetée sur le plan tangent au milieu.
fn chord_normal(low: Vec3, high: Vec3) -> Vec3 {
    let mid = normalize(add(low, high));
    let chord = sub(high, low);
    safe_normalize(sub(chord, scale(mid, dot(chord, mid))))
}

/// Repères locaux (est, nord) : est = c × ŷ, nord = ŷ − sinφ·c, normalisés ; nuls au pôle exact.
fn local_frames(centers: &[Vec3]) -> (Vec<Vec3>, Vec<Vec3>) {
    let pole = [0.0, 1.0, 0.0];
    centers
        .iter()
        .map(|c| {
            let east = safe_normalize(cross(*c, pole));
            let north = safe_normalize(sub(pole, scale(*c, c[1])));
            (east, north)
        })
        .unzip()
}
