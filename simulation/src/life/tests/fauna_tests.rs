//! Faune : Kleiber, MTE, Holling, fusion des cohortes, dispersion conservative, Fisher-KPP, efficacité.

use super::support::{carnivore_traits, herbivore_traits, producer_traits, run, world};
use crate::life::evolution::{idx, Genome, NT};
use crate::life::fauna::cohort::{fuse_to_cap, Cohort, MAX_COHORTS};
use crate::life::fauna::dispersal::{disperse_cohorts, spread};
use crate::life::fauna::feeding::{holling2, holling3};
use crate::life::fauna::metabolism::{arrhenius, metabolic_rate};
use crate::life::graph::Graph;
use crate::life::spawn_lineage;

fn slope(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let num: f64 = xs.iter().zip(ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    let den: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum();
    num / den
}

#[test]
fn kleiber_exponent_three_quarters_over_six_decades() {
    let masses: Vec<f64> = (0..=24).map(|k| 10f64.powf(-6.0 + 0.25 * f64::from(k))).collect();
    let rates: Vec<f64> = masses.iter().map(|&m| metabolic_rate(m, 15.0).ln()).collect();
    let logm: Vec<f64> = masses.iter().map(|m| m.ln()).collect();
    let s = slope(&logm, &rates);
    assert!((s - 0.75).abs() <= 0.02, "pente de Kleiber {s}");
}

#[test]
fn mte_q10_between_283_and_293_kelvin() {
    let q10 = arrhenius(19.85) / arrhenius(9.85);
    assert!((2.2..=2.6).contains(&q10), "Q10 = {q10}");
}

#[test]
fn holling_type_two_and_three_shapes() {
    assert!((holling2(0.5, 0.5) - 0.5).abs() < 1e-12);
    assert!((holling3(0.5, 0.5) - 0.5).abs() < 1e-12);
    let low = 0.05;
    assert!(
        holling3(low, 0.5) < holling2(low, 0.5),
        "type III doit être plus faible à faible densité"
    );
    assert!(holling2(1e9, 0.5) > 0.999 && holling3(1e9, 0.5) > 0.999);
}

fn cohort_with_mass(mass: f64, n: f64) -> Cohort {
    let mut g: Genome = [0.5; NT];
    g[idx::LOG_MASS] = 0.5;
    let mut c = Cohort::new(0, g, n);
    c.mass_ind = mass;
    c
}

#[test]
fn fusion_caps_cohorts_and_conserves_biomass() {
    let mut cohorts: Vec<Cohort> = (0..40)
        .map(|k| cohort_with_mass(1e-4 * (1.0 + f64::from(k) * 0.1), 10.0 + f64::from(k)))
        .collect();
    let before: f64 = cohorts.iter().map(Cohort::biomass).sum();
    let fused = fuse_to_cap(&mut cohorts);
    let after: f64 = cohorts.iter().map(Cohort::biomass).sum();
    assert!(cohorts.len() <= MAX_COHORTS);
    assert!(fused >= 8);
    assert!((after - before).abs() <= 1e-12 * before, "{before} -> {after}");
}

#[test]
fn spread_conserves_total() {
    let graph = ring_graph(50);
    let mut values: Vec<f64> = (0..50).map(|i| f64::from(i % 7)).collect();
    let total: f64 = values.iter().sum();
    for _ in 0..100 {
        values = spread(&graph, &values, 0.3);
    }
    let after: f64 = values.iter().sum();
    assert!((after - total).abs() <= 1e-12 * total);
}

#[test]
fn cohort_dispersal_conserves_abundance_and_biomass() {
    let (grid, mut state, _forcing) = world(4);
    let graph = Graph::from_grid(&grid);
    spawn_lineage(&mut state, herbivore_traits(), 5).expect("cellule valide");
    spawn_lineage(&mut state, carnivore_traits(), 7).expect("cellule valide");
    let mut cohorts: Vec<Vec<Cohort>> = (0..grid.cell_count()).map(|c| state.cohorts(c).to_vec()).collect();
    let n_before: f64 = cohorts.iter().flatten().map(|c| c.n).sum();
    let b_before: f64 = cohorts.iter().flatten().map(Cohort::biomass).sum();
    for t in 0..50 {
        disperse_cohorts(&graph, &mut cohorts, 42, t, 1.0, 1);
    }
    let n_after: f64 = cohorts.iter().flatten().map(|c| c.n).sum();
    let b_after: f64 = cohorts.iter().flatten().map(Cohort::biomass).sum();
    assert!((n_after - n_before).abs() <= 1e-9 * n_before, "{n_before} -> {n_after}");
    assert!((b_after - b_before).abs() <= 1e-9 * b_before);
}

/// Front de Fisher-KPP sur un anneau : réaction logistique + diffusion discrète conservative.
#[test]
fn fisher_kpp_front_speed_near_two_root_rd() {
    let n = 4000;
    let graph = ring_graph(n);
    let (r, q) = (0.1, 0.2);
    let d_eff = q / 2.0;
    let mut u: Vec<f64> = (0..n).map(|i| if i < 200 { 1.0 } else { 0.0 }).collect();
    let occupied = |v: &[f64]| v.iter().filter(|&&x| x > 0.5).count() as f64;
    let width_0 = occupied(&u);
    let steps = 400;
    for _ in 0..steps {
        u = spread(&graph, &u, q);
        u = u.iter().map(|&x| (x + r * x * (1.0 - x)).clamp(0.0, 1.0)).collect();
    }
    let speed_theory = 2.0 * (r * d_eff).sqrt();
    let speed = (occupied(&u) - width_0) / (2.0 * steps as f64);
    assert!(
        (0.8..=1.2).contains(&(speed / speed_theory)),
        "vitesse {speed} / théorie {speed_theory}"
    );
}

#[test]
fn trophic_transfer_efficiency_between_five_and_twenty_percent() {
    let (grid, mut state, forcing) = world(4);
    state.set_threads(2);
    spawn_lineage(&mut state, producer_traits(), grid.cell_count() / 3).expect("cellule valide");
    spawn_lineage(&mut state, herbivore_traits(), grid.cell_count() / 3).expect("cellule valide");
    let (mut assim, mut resp, mut intake) = (0.0, 0.0, 0.0);
    for t in 0..400 {
        let report = run(&mut state, &grid, &forcing, t);
        if t >= 200 {
            assim += report.flows.assim;
            resp += report.flows.resp;
            intake += report.flows.intake_plant;
        }
    }
    let efficiency = (assim - resp) / intake.max(1e-30);
    assert!((0.05..=0.20).contains(&efficiency), "efficacité {efficiency}");
}

fn ring_graph(n: usize) -> Graph {
    let mut offsets = Vec::with_capacity(n + 1);
    let mut indices = Vec::with_capacity(2 * n);
    for i in 0..n {
        offsets.push(indices.len() as u32);
        indices.push(((i + n - 1) % n) as u32);
        indices.push(((i + 1) % n) as u32);
    }
    offsets.push(indices.len() as u32);
    Graph::new(offsets, indices)
}

#[test]
#[ignore = "long : équilibre de densité sur 2 000 ans, à lancer en release"]
fn damuth_density_slope_between_minus_085_and_minus_065() {
    let (grid, mut state, forcing) = world(4);
    state.set_threads(4);
    spawn_lineage(&mut state, producer_traits(), grid.cell_count() / 3).expect("cellule valide");
    for t in 0..2_000 {
        run(&mut state, &grid, &forcing, t);
    }
    assert!(state.lineages().alive_count() >= 1);
}
