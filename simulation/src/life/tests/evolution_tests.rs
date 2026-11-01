//! Évolution : mutation bornée, identifiants et phylogénie, noms, extinction, spéciation allopatrique.

use super::support::{grid, herbivore_traits, uniform_traits};
use crate::life::evolution::extinction::{update_extinctions, EXTINCTION_YEARS, MIN_POPULATION};
use crate::life::evolution::genome::{genetic_distance, idx, mutate, Genome, NT};
use crate::life::evolution::lineage::LineageTable;
use crate::life::evolution::speciation::{speciate, ISOLATION_YEARS};
use crate::life::fauna::cohort::Cohort;
use crate::life::graph::Graph;
use crate::life::naming::species_name;
use crate::shared::rng::unit_f64;
use std::collections::HashSet;

#[test]
fn mutation_stays_in_unit_interval() {
    let mut parent = uniform_traits(0.0);
    for k in 0..20_000_u64 {
        parent[(k % NT as u64) as usize] = unit_f64(9, k, 1, 2);
        let child = mutate(&parent, 9, k, 3, 17);
        assert!(child.iter().all(|&x| (0.0..=1.0).contains(&x)), "{child:?}");
        parent = child;
    }
}

#[test]
fn lineage_ids_are_sequential_and_phylogeny_recorded() {
    let mut table = LineageTable::default();
    let a = table.create(herbivore_traits(), None, 0, 4);
    let b = table.create(herbivore_traits(), Some(a), 10, 5);
    assert_eq!((a, b), (0, 1));
    assert_eq!(table.phylogeny().len(), 2);
    assert_eq!(table.phylogeny()[1].parent, Some(a));
    assert_eq!(table.alive_count(), 2);
}

#[test]
fn species_names_are_deterministic_and_mostly_distinct() {
    let names: Vec<String> = (0..1000).map(species_name).collect();
    let again: Vec<String> = (0..1000).map(species_name).collect();
    assert_eq!(names, again);
    let distinct: HashSet<&String> = names.iter().collect();
    assert!(distinct.len() >= 950, "{} noms distincts", distinct.len());
    assert!(names[0].chars().next().is_some_and(char::is_uppercase));
}

#[test]
fn lineage_goes_extinct_after_fifty_years_below_threshold() {
    let mut table = LineageTable::default();
    table.create(herbivore_traits(), None, 0, 0);
    table.get_mut(0).expect("lignée créée").n_total = MIN_POPULATION / 2.0;
    assert!(update_extinctions(&mut table, EXTINCTION_YEARS - 1.0, 1).is_empty());
    assert!(table.is_alive(0));
    assert_eq!(update_extinctions(&mut table, 1.0, 2), vec![0]);
    assert!(!table.is_alive(0));
}

#[test]
fn allopatric_split_after_isolation_with_genetic_divergence() {
    let g = grid(4);
    let graph = Graph::from_grid(&g);
    let (block_a, block_b) = two_blocks(&graph, 6);
    let mut table = LineageTable::default();
    let founder = table.create(herbivore_traits(), None, 0, block_a[0] as u32);
    let mut cohorts: Vec<Vec<Cohort>> = vec![Vec::new(); graph.cell_count()];
    for &c in &block_a {
        cohorts[c].push(cohort(founder, herbivore_traits(), 1.0));
    }
    let mut diverged = herbivore_traits();
    diverged[idx::GREGARIOUS] = 0.9;
    diverged[idx::RECOMB] = 0.1;
    for &c in &block_b {
        cohorts[c].push(cohort(founder, diverged, 1.0));
    }
    assert!(genetic_distance(&herbivore_traits(), &diverged) > 0.05);
    assert!(speciate(&mut table, &mut cohorts, &graph, 0, ISOLATION_YEARS / 2.0).is_empty());
    let new = speciate(&mut table, &mut cohorts, &graph, 1, ISOLATION_YEARS / 2.0);
    assert_eq!(new.len(), 1, "une scission attendue après l'isolement");
    assert_eq!(table.get(new[0]).expect("lignée fille").parent, Some(founder));
    assert!(cohorts[block_b[0]].iter().all(|c| c.lineage == new[0]));
    assert!(cohorts[block_a[0]].iter().all(|c| c.lineage == founder));
}

/// Deux blocs de cellules disjoints (aucune arête entre eux) : des segments de l'anneau de la grille.
fn two_blocks(graph: &Graph, size: usize) -> (Vec<usize>, Vec<usize>) {
    let mut a = Vec::new();
    let mut b = Vec::new();
    let mut seen = vec![false; graph.cell_count()];
    let mut frontier = vec![0_usize];
    while a.len() < size && !frontier.is_empty() {
        let c = frontier.remove(0);
        if seen[c] {
            continue;
        }
        seen[c] = true;
        a.push(c);
        frontier.extend(graph.neighbors(c).iter().map(|&n| n as usize));
    }
    let far = graph.cell_count() - 1;
    let mut frontier = vec![far];
    while b.len() < size && !frontier.is_empty() {
        let c = frontier.remove(0);
        if seen[c] || a.contains(&c) {
            continue;
        }
        seen[c] = true;
        b.push(c);
        frontier.extend(graph.neighbors(c).iter().map(|&n| n as usize));
    }
    (a, b)
}

fn cohort(lineage: u32, genome: Genome, n: f64) -> Cohort {
    let mut c = Cohort::new(lineage, genome, n);
    c.mass_ind = c.mass_adult;
    c
}
