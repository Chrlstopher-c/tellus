//! Cohorte : individus d'une même lignée dans une cellule. Fusion au plafond, sans perte de biomasse.

use crate::life::evolution::{adult_mass_kgc, birth_fraction, genetic_distance, idx, Genome, LineageId};

/// Plafond de cohortes par cellule ; au-delà, fusion des paires les plus proches (vivant.md § 4.2).
pub const MAX_COHORTS: usize = 32;
const DUST_N: f64 = 1e-9;
/// Écart sous lequel deux cohortes de même lignée sont fusionnées à l'insertion.
const MERGE_RATIO: f64 = 1.122;
const PRODUCER_THRESHOLD: f64 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cohort {
    pub lineage: LineageId,
    /// Effectif (individus par cellule).
    pub n: f64,
    /// Masse individuelle, kgC.
    pub mass_ind: f64,
    /// Masse adulte, kgC (dérivée du génome à la naissance).
    pub mass_adult: f64,
    /// Masse à la naissance, kgC.
    pub mass_birth: f64,
    pub age_yr: f64,
    pub genome: Genome,
}

impl Cohort {
    /// Cohorte d'adultes de la lignée `lineage` : la masse individuelle démarre à la masse adulte.
    pub fn new(lineage: LineageId, genome: Genome, n: f64) -> Self {
        let mass_adult = adult_mass_kgc(&genome);
        Self {
            lineage,
            n,
            mass_ind: mass_adult,
            mass_adult,
            mass_birth: mass_adult * birth_fraction(&genome),
            age_yr: 0.0,
            genome,
        }
    }

    pub fn biomass(&self) -> f64 {
        self.n * self.mass_ind
    }

    /// Lignée photosynthétique : présence de la flore, biomasse portée par le modèle de flore.
    pub fn is_producer(&self) -> bool {
        self.genome[idx::PHOTO] >= PRODUCER_THRESHOLD
    }
}

/// Fusion de deux cohortes d'une même lignée : biomasse et effectif conservés exactement.
pub fn merge(a: &Cohort, b: &Cohort) -> Cohort {
    let n = a.n + b.n;
    if n <= 0.0 {
        return *a;
    }
    let (wa, wb) = (a.n / n, b.n / n);
    let mut genome = a.genome;
    for (i, value) in genome.iter_mut().enumerate() {
        *value = a.genome[i] * wa + b.genome[i] * wb;
    }
    Cohort {
        lineage: a.lineage,
        n,
        mass_ind: (a.biomass() + b.biomass()) / n,
        mass_adult: a.mass_adult * wa + b.mass_adult * wb,
        mass_birth: a.mass_birth * wa + b.mass_birth * wb,
        age_yr: a.age_yr * wa + b.age_yr * wb,
        genome,
    }
}

/// Écart entre deux cohortes de même lignée : log10 des masses individuelles + distance génétique.
fn pair_distance(a: &Cohort, b: &Cohort) -> f64 {
    let mass_gap = (a.mass_ind.max(1e-300).log10() - b.mass_ind.max(1e-300).log10()).abs();
    mass_gap + genetic_distance(&a.genome, &b.genome)
}

/// Rapport des masses individuelles (≥ 1), sans logarithme : coût négligeable dans la boucle d'insertion.
fn mass_ratio(a: f64, b: f64) -> f64 {
    let (hi, lo) = (a.max(b), a.min(b).max(1e-300));
    hi / lo
}

/// Insère une cohorte entrante : fusion avec la plus proche de même lignée si elle est voisine ou si la cellule
/// est pleine, sinon ajout. Évite l'accumulation de cohortes quasi identiques entre deux fusions.
pub fn insert(cohorts: &mut Vec<Cohort>, incoming: Cohort) {
    if incoming.n <= 0.0 {
        return;
    }
    let nearest = cohorts
        .iter()
        .enumerate()
        .filter(|(_, c)| c.lineage == incoming.lineage)
        .map(|(k, c)| (k, mass_ratio(c.mass_ind, incoming.mass_ind)))
        .min_by(|a, b| a.1.total_cmp(&b.1));
    match nearest {
        Some((k, ratio)) if ratio < MERGE_RATIO || cohorts.len() >= MAX_COHORTS => {
            cohorts[k] = merge(&cohorts[k], &incoming);
        }
        _ => cohorts.push(incoming),
    }
}

/// Paire de cohortes de même lignée la plus proche (masse logarithmique + distance génétique).
fn closest_pair(cohorts: &[Cohort]) -> Option<(usize, usize)> {
    let mut best: Option<(f64, usize, usize)> = None;
    for i in 0..cohorts.len() {
        for j in (i + 1)..cohorts.len() {
            let (a, b) = (&cohorts[i], &cohorts[j]);
            if a.lineage != b.lineage {
                continue;
            }
            let distance = pair_distance(a, b);
            if best.is_none_or(|(d, _, _)| distance < d) {
                best = Some((distance, i, j));
            }
        }
    }
    best.map(|(_, i, j)| (i, j))
}

/// Ramène la cellule sous le plafond par fusions successives ; renvoie le nombre de fusions.
pub fn fuse_to_cap(cohorts: &mut Vec<Cohort>) -> u32 {
    let mut fusions = 0;
    while cohorts.len() > MAX_COHORTS {
        let Some((i, j)) = closest_pair(cohorts) else {
            break;
        };
        cohorts[j] = merge(&cohorts[i], &cohorts[j]);
        cohorts.remove(i);
        fusions += 1;
    }
    fusions
}

/// Retire les cohortes sous le seuil de poussière ; renvoie leur biomasse (à reverser à la litière).
pub fn take_dust(cohorts: &mut Vec<Cohort>) -> f64 {
    let mut lost = 0.0;
    cohorts.retain(|c| {
        if c.n > DUST_N {
            true
        } else {
            lost += c.biomass();
            false
        }
    });
    lost
}

/// Retire toutes les cohortes des lignées `ids` (extinction) ; renvoie la biomasse retirée.
pub fn remove_lineages(cohorts: &mut Vec<Cohort>, ids: &[LineageId]) -> f64 {
    let mut lost = 0.0;
    cohorts.retain(|c| {
        if ids.contains(&c.lineage) {
            lost += c.biomass();
            false
        } else {
            true
        }
    });
    lost
}
