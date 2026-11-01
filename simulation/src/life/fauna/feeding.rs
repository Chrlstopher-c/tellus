//! Consommation : Holling II (herbivores, plantes), Holling III (prédateurs, proies animales), gaussienne de taille.

use super::cohort::Cohort;
use super::metabolism::max_intake_at;
use crate::life::evolution::idx;

/// Demi-saturation des plantes accessibles par cellule, kgC.
pub const HALF_SAT_PLANT_KGC: f64 = 500.0;
/// Demi-saturation des proies animales pondérées par cellule, kgC.
pub const HALF_SAT_ANIMAL_KGC: f64 = 50.0;
/// Taille de proie optimale : 10 fois plus petite que le prédateur (log10 du rapport).
const PREY_OPTIMUM_LOG10: f64 = -1.0;
const PREY_WIDTH_LOG10: f64 = 1.0;

/// Réponse fonctionnelle de type II : saturation par le temps de manipulation.
pub fn holling2(n: f64, half: f64) -> f64 {
    if n.is_nan() || n <= 0.0 {
        return 0.0;
    }
    if n.is_infinite() {
        return 1.0;
    }
    n / (half + n)
}

/// Réponse fonctionnelle de type III : seuil sigmoïde, refuges à faible densité.
pub fn holling3(n: f64, half: f64) -> f64 {
    if n.is_nan() || n <= 0.0 {
        return 0.0;
    }
    if n.is_infinite() {
        return 1.0;
    }
    let n2 = n * n;
    n2 / (half * half + n2)
}

/// Préférence pour une proie de masse `prey` selon la masse du prédateur (gaussienne en log-masse).
pub fn prey_weight(prey_mass: f64, pred_mass: f64) -> f64 {
    let x = (prey_mass.max(1e-300) / pred_mass.max(1e-300)).log10() - PREY_OPTIMUM_LOG10;
    (-x * x / (2.0 * PREY_WIDTH_LOG10 * PREY_WIDTH_LOG10)).exp()
}

/// Proies animales accessibles au prédateur `pred` : biomasse pondérée totale et part de chaque cohorte.
pub fn animal_prey(cohorts: &[Cohort], pred: usize) -> (f64, Vec<(usize, f64)>) {
    let predator = &cohorts[pred];
    let mut total = 0.0;
    let mut list = Vec::new();
    for (j, prey) in cohorts.iter().enumerate() {
        if j == pred || prey.lineage == predator.lineage || prey.is_producer() {
            continue;
        }
        let weight = prey_weight(prey.mass_ind, predator.mass_ind) * prey.biomass();
        if weight > 0.0 {
            total += weight;
            list.push((j, weight));
        }
    }
    (total, list)
}

/// Prélève les proies animales : biomasse retirée des cohortes-proies, répartie au prorata du poids.
fn take_animals(cohorts: &mut [Cohort], pred: usize, demand: f64) -> f64 {
    let (total, list) = animal_prey(cohorts, pred);
    let take = (demand * holling3(total, HALF_SAT_ANIMAL_KGC)).min(total);
    if take <= 0.0 {
        return 0.0;
    }
    let mut removed = 0.0;
    for (j, weight) in list {
        let prey = &mut cohorts[j];
        let share = take * weight / total;
        let killed = (share / prey.mass_ind.max(1e-300)).min(prey.n);
        prey.n -= killed;
        removed += killed * prey.mass_ind;
    }
    removed
}

/// Consommation de toutes les cohortes d'une cellule pendant `h` ans.
/// Renvoie, par cohorte, (plantes prélevées, proies animales prélevées) en kgC ; `plant_avail` est décrémenté.
pub fn feed(cohorts: &mut [Cohort], plant_avail: &mut f64, h: f64, arr: f64) -> Vec<(f64, f64)> {
    let mut taken = vec![(0.0, 0.0); cohorts.len()];
    for i in 0..cohorts.len() {
        let cohort = cohorts[i];
        if cohort.is_producer() || cohort.n <= 0.0 {
            continue;
        }
        let demand = cohort.n * max_intake_at(cohort.mass_ind, arr) * h;
        let diet = cohort.genome[idx::DIET].clamp(0.0, 1.0);
        let plant_req = demand * (1.0 - diet) * holling2(*plant_avail, HALF_SAT_PLANT_KGC);
        let plant = plant_req.min(*plant_avail).max(0.0);
        *plant_avail -= plant;
        let animal = if diet > 0.0 {
            take_animals(cohorts, i, demand * diet)
        } else {
            0.0
        };
        taken[i] = (plant, animal);
    }
    taken
}
