//! Dispersion conservative : diffusion locale sur le graphe et sauts à queue épaisse (vivant.md § 3.6).

use super::cohort::{fuse_to_cap, insert, Cohort};
use crate::life::evolution::mobility_rate;
use crate::life::graph::Graph;
use crate::shared::rng::unit_f64;

/// Part de chaque départ qui fait un saut long (p(L) ∝ L⁻² sur le nombre de pas du marcheur).
const LONG_SHARE: f64 = 0.05;
const MAX_JUMP_HOPS: u32 = 16;
const MAX_FRACTION: f64 = 0.5;

/// Diffusion scalaire conservative : chaque cellule envoie `q` de sa valeur, réparti sur ses voisins.
pub fn spread(graph: &Graph, values: &[f64], q: f64) -> Vec<f64> {
    let mut next = values.to_vec();
    for (i, &value) in values.iter().enumerate() {
        let out = q * value;
        let neighbours = graph.neighbors(i);
        if neighbours.is_empty() {
            continue;
        }
        next[i] -= out;
        let share = out / neighbours.len() as f64;
        for &j in neighbours {
            next[j as usize] += share;
        }
    }
    next
}

/// Déplace une fraction de chaque cohorte vers les cellules voisines et lointaines ; biomasse conservée.
/// Seules les cellules en phase (`(cell + tick) % period == 0`) envoient des départs, sur un pas `span`.
pub fn disperse_cohorts(graph: &Graph, cohorts: &mut [Vec<Cohort>], seed: u64, tick: u64, span: f64, period: u64) {
    let mut arrivals: Vec<(usize, Cohort)> = Vec::new();
    for (cell, own) in cohorts.iter_mut().enumerate() {
        if !(cell as u64 + tick).is_multiple_of(period) {
            continue;
        }
        for (slot, cohort) in own.iter_mut().enumerate() {
            if let Some(parcel) = departure(cohort, span) {
                route(graph, cell, parcel, (seed, tick, slot as u32), &mut arrivals);
            }
        }
    }
    for (cell, parcel) in arrivals {
        insert(&mut cohorts[cell], parcel);
    }
    for cell in cohorts.iter_mut() {
        fuse_to_cap(cell);
    }
}

/// Retire de la cohorte source la fraction qui part ; renvoie la cohorte partante, ou `None` si rien ne bouge.
fn departure(cohort: &mut Cohort, span: f64) -> Option<Cohort> {
    let q = (mobility_rate(&cohort.genome) * span).min(MAX_FRACTION);
    let out = cohort.n * q;
    if out <= 0.0 {
        return None;
    }
    cohort.n -= out;
    Some(Cohort { n: out, ..*cohort })
}

fn route(
    graph: &Graph,
    cell: usize,
    parcel: Cohort,
    (seed, tick, slot): (u64, u64, u32),
    arrivals: &mut Vec<(usize, Cohort)>,
) {
    let neighbours = graph.neighbors(cell);
    let long = parcel.n * LONG_SHARE;
    let local = parcel.n - long;
    if neighbours.is_empty() {
        arrivals.push((cell, Cohort { n: local, ..parcel }));
    } else {
        let share = local / neighbours.len() as f64;
        for &j in neighbours {
            arrivals.push((j as usize, Cohort { n: share, ..parcel }));
        }
    }
    let end = walk(graph, cell, seed, tick, slot);
    arrivals.push((end, Cohort { n: long, ..parcel }));
}

/// Saut long : marche aléatoire de L pas, L ≈ 1/u (queue en L⁻²), plafonnée.
fn walk(graph: &Graph, start: usize, seed: u64, tick: u64, slot: u32) -> usize {
    let u = unit_f64(seed, tick, start as u32, slot.wrapping_mul(2).wrapping_add(1)).max(1e-12);
    let hops = (1.0 / u).floor().clamp(1.0, f64::from(MAX_JUMP_HOPS)) as u32;
    let mut cell = start;
    for hop in 0..hops {
        let neighbours = graph.neighbors(cell);
        if neighbours.is_empty() {
            break;
        }
        let r = unit_f64(
            seed,
            tick,
            cell as u32,
            0x1000 + slot.wrapping_mul(32).wrapping_add(hop),
        );
        let pick = ((r * neighbours.len() as f64) as usize).min(neighbours.len() - 1);
        cell = neighbours[pick] as usize;
    }
    cell
}
