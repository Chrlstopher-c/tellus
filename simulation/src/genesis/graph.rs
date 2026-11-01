//! Croissance multi-source (Dijkstra) sur le graphe des cellules : distances en mètres, origine par source.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::vec3::angle;
use crate::shared::grid::Grid;

/// Marque une cellule jamais atteinte par la croissance.
pub(super) const UNREACHED: u32 = u32::MAX;

/// Distance au plus proche germe et indice de ce germe (dans la liste fournie à `grow`).
pub(super) struct Field {
    pub(super) dist: Vec<f64>,
    pub(super) origin: Vec<u32>,
}

#[derive(PartialEq)]
struct Node {
    dist: f64,
    cell: u32,
}

impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .dist
            .total_cmp(&self.dist)
            .then_with(|| other.cell.cmp(&self.cell))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Arc de grand cercle entre deux cellules, en mètres.
pub(super) fn arc_m(grid: &Grid, a: usize, b: usize) -> f64 {
    let centers = grid.centers();
    angle(centers[a], centers[b]) * grid.radius_m()
}

/// Croissance depuis `seeds` (la source `k` a l'origine `k`). `step(from, to)` donne le coût d'entrée
/// dans `to`, ou `None` si la cellule est interdite à cette croissance.
pub(super) fn grow(grid: &Grid, seeds: &[usize], step: impl Fn(usize, usize) -> Option<f64>) -> Field {
    let count = grid.cell_count();
    let mut field = Field {
        dist: vec![f64::INFINITY; count],
        origin: vec![UNREACHED; count],
    };
    let mut heap = BinaryHeap::new();
    for (k, &seed) in seeds.iter().enumerate() {
        field.dist[seed] = 0.0;
        field.origin[seed] = k as u32;
        heap.push(Node {
            dist: 0.0,
            cell: seed as u32,
        });
    }
    while let Some(Node { dist, cell }) = heap.pop() {
        relax_neighbours(grid, &mut field, &mut heap, (cell as usize, dist), &step);
    }
    field
}

/// Relaxe les voisins de `(cell, dist)` : chacun garde la plus petite distance et l'origine de son meilleur chemin.
fn relax_neighbours(
    grid: &Grid,
    field: &mut Field,
    heap: &mut BinaryHeap<Node>,
    (cell, dist): (usize, f64),
    step: &impl Fn(usize, usize) -> Option<f64>,
) {
    if dist > field.dist[cell] {
        return;
    }
    for &next in grid.neighbors(cell) {
        let next = next as usize;
        let Some(cost) = step(cell, next) else {
            continue;
        };
        let candidate = dist + cost;
        if candidate < field.dist[next] {
            field.dist[next] = candidate;
            field.origin[next] = field.origin[cell];
            heap.push(Node {
                dist: candidate,
                cell: next as u32,
            });
        }
    }
}
