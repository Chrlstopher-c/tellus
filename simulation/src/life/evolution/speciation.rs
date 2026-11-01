//! Spéciation allopatrique : une lignée isolée géographiquement et génétiquement divergente se scinde.

use super::genome::{genetic_distance, Genome, NT};
use super::lineage::{LineageId, LineageTable};
use crate::life::fauna::Cohort;
use crate::life::graph::Graph;

/// Isolement minimal entre deux sous-populations avant scission, années (vivant.md § 6.2).
pub const ISOLATION_YEARS: f64 = 1_000.0;
/// Distance génétique minimale (RMS sur les traits de compatibilité) pour une scission.
pub const SPLIT_DISTANCE: f64 = 0.05;
const OCCUPIED_N: f64 = 1e-6;

/// Teste chaque lignée vivante ; renvoie les identifiants des lignées filles créées.
pub fn speciate(
    table: &mut LineageTable,
    cohorts: &mut [Vec<Cohort>],
    graph: &Graph,
    tick: u64,
    check_years: f64,
) -> Vec<LineageId> {
    let mut created = Vec::new();
    for id in table.alive_ids() {
        if let Some(child) = try_split(table, cohorts, graph, id, tick, check_years) {
            created.push(child);
        }
    }
    created
}

fn try_split(
    table: &mut LineageTable,
    cohorts: &mut [Vec<Cohort>],
    graph: &Graph,
    id: LineageId,
    tick: u64,
    check_years: f64,
) -> Option<LineageId> {
    let parts = components(cohorts, graph, id);
    let lineage = table.get_mut(id)?;
    if parts.len() < 2 {
        lineage.isolation_yr = 0.0;
        return None;
    }
    lineage.isolation_yr += check_years;
    if lineage.isolation_yr < ISOLATION_YEARS {
        return None;
    }
    let (main, rest) = parts.split_first()?;
    let (main_mean, _) = mean_genome(cohorts, id, main);
    for part in rest {
        let (mean, _) = mean_genome(cohorts, id, part);
        if genetic_distance(&mean, &main_mean) > SPLIT_DISTANCE {
            let child = table.create(mean, Some(id), tick, part[0] as u32);
            move_cells(cohorts, id, part, child);
            if let Some(parent) = table.get_mut(id) {
                parent.isolation_yr = 0.0;
            }
            return Some(child);
        }
    }
    None
}

/// Composantes connexes des cellules occupées par la lignée ; la première contient la plus petite cellule.
fn components(cohorts: &[Vec<Cohort>], graph: &Graph, id: LineageId) -> Vec<Vec<usize>> {
    let occupied: Vec<bool> = cohorts
        .iter()
        .map(|cell| cell.iter().any(|c| c.lineage == id && c.n > OCCUPIED_N))
        .collect();
    let mut seen = vec![false; cohorts.len()];
    let mut parts = Vec::new();
    for start in 0..cohorts.len() {
        if occupied[start] && !seen[start] {
            parts.push(flood(graph, &occupied, &mut seen, start));
        }
    }
    parts
}

fn flood(graph: &Graph, occupied: &[bool], seen: &mut [bool], start: usize) -> Vec<usize> {
    let mut part = Vec::new();
    let mut stack = vec![start];
    seen[start] = true;
    while let Some(cell) = stack.pop() {
        part.push(cell);
        for &next in graph.neighbors(cell) {
            let next = next as usize;
            if occupied[next] && !seen[next] {
                seen[next] = true;
                stack.push(next);
            }
        }
    }
    part.sort_unstable();
    part
}

/// Moyenne des traits d'une lignée sur un ensemble de cellules, pondérée par l'effectif.
fn mean_genome(cohorts: &[Vec<Cohort>], id: LineageId, cells: &[usize]) -> (Genome, f64) {
    let mut sum = [0.0; NT];
    let mut total = 0.0;
    for &cell in cells {
        for c in cohorts[cell].iter().filter(|c| c.lineage == id) {
            total += c.n;
            for (s, g) in sum.iter_mut().zip(c.genome.iter()) {
                *s += c.n * g;
            }
        }
    }
    if total > 0.0 {
        for s in sum.iter_mut() {
            *s /= total;
        }
    }
    (sum, total)
}

fn move_cells(cohorts: &mut [Vec<Cohort>], id: LineageId, cells: &[usize], child: LineageId) {
    for &cell in cells {
        for c in cohorts[cell].iter_mut().filter(|c| c.lineage == id) {
            c.lineage = child;
        }
    }
}
