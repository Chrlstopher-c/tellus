//! Mise à jour animale d'une cellule : sous-pas de consommation et de métabolisme ; reproduction mise en réserve
//! puis libérée en fin de pas (un nouveau-né par cohorte et par pas, génome muté une seule fois).

use super::cohort::{fuse_to_cap, insert, take_dust, Cohort};
use super::damuth::territorial_cap;
use super::feeding::feed;
use super::metabolism::{arrhenius, metabolic_rate_at, mortality_at};
use crate::life::evolution::{longevity_yr, mutate, repro_fraction, Genome};

const SUBSTEP_YR: f64 = 1.0;
const MAX_SUBSTEPS: u32 = 64;
const ASSIM_PLANT: f64 = 0.4;
const ASSIM_ANIMAL: f64 = 0.8;
const SENESCENCE_RATE: f64 = 0.5;
const STARVATION_RATE: f64 = 0.5;
const CROWD_RATE: f64 = 10.0;
/// Capacité de charge animale : biomasse animale maximale, fraction de la biomasse végétale de la cellule.
const CARRY_FRACTION: f64 = 0.3;
/// Fraction comestible de la biomasse végétale (feuilles, herbes, plancton), disponible pour la consommation.
const EDIBLE_FRACTION: f64 = 0.3;
/// Mortalité territoriale par an et par excès relatif au plafond de Damuth.
const TERRITORY_RATE: f64 = 10.0;
/// En deçà, une cohorte est une poussière : elle ne se nourrit plus et sa biomasse rejoint la litière.
const ACTIVE_N: f64 = 1e-9;

/// Contexte d'une cellule pour un pas : température, biomasse végétale, aire et aléa à compteur.
#[derive(Debug, Clone, Copy)]
pub struct FaunaEnv {
    pub temp_c: f64,
    pub plant_kgc: f64,
    pub area_m2: f64,
    pub seed: u64,
    pub tick: u64,
    pub cell: u32,
}

/// Flux d'une cellule sur un pas, kgC. `dead` et `egest` vont à la litière, `resp` hors du vivant.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FaunaFlux {
    pub intake_plant: f64,
    pub intake_animal: f64,
    pub assim: f64,
    pub resp: f64,
    pub egest: f64,
    pub dead: f64,
    pub fusions: u32,
    pub clamps: u32,
}

struct StepCtx<'a> {
    env: &'a FaunaEnv,
    arr: f64,
    h: f64,
    crowd: f64,
}

/// Fait avancer les cohortes d'une cellule de `span` ans. `intake_plant` est prélevé sur la biomasse végétale.
pub fn update_cell(cohorts: &mut Vec<Cohort>, env: &FaunaEnv, span: f64) -> FaunaFlux {
    let mut flux = FaunaFlux::default();
    if cohorts.is_empty() || span <= 0.0 {
        return flux;
    }
    let steps = ((span / SUBSTEP_YR).ceil() as u32).clamp(1, MAX_SUBSTEPS);
    let h = span / f64::from(steps);
    let arr = arrhenius(env.temp_c);
    let mut plant_avail = EDIBLE_FRACTION * env.plant_kgc;
    let genomes: Vec<Genome> = cohorts
        .iter()
        .enumerate()
        .map(|(k, c)| mutate(&c.genome, env.seed, env.tick, env.cell, k as u32))
        .collect();
    let mut reserve = vec![0.0; cohorts.len()];
    for _ in 0..steps {
        substep(cohorts, env, arr, h, &mut plant_avail, &mut reserve, &mut flux);
        release_offspring(cohorts, &reserve, &genomes);
        reserve.iter_mut().for_each(|r| *r = 0.0);
    }
    flux.dead += take_dust(cohorts);
    flux.fusions += fuse_to_cap(cohorts);
    flux
}

fn substep(
    cohorts: &mut [Cohort],
    env: &FaunaEnv,
    arr: f64,
    h: f64,
    plant_avail: &mut f64,
    reserve: &mut Vec<f64>,
    flux: &mut FaunaFlux,
) {
    reserve.resize(cohorts.len(), 0.0);
    let taken = feed(cohorts, plant_avail, h, arr);
    let ctx = StepCtx {
        env,
        arr,
        h,
        crowd: crowding(cohorts, env.plant_kgc),
    };
    for (k, cohort) in cohorts.iter_mut().enumerate() {
        if cohort.is_producer() || cohort.n <= ACTIVE_N {
            continue;
        }
        let (plant, animal) = taken[k];
        flux.intake_plant += plant;
        flux.intake_animal += animal;
        reserve[k] += living_step(cohort, (plant, animal), &ctx, flux);
    }
}

/// Libère la réserve de reproduction du sous-pas : nouveau-nés au génome muté une fois par pas (`genomes`).
fn release_offspring(cohorts: &mut Vec<Cohort>, reserve: &[f64], genomes: &[Genome]) {
    let children: Vec<Cohort> = reserve
        .iter()
        .enumerate()
        .filter(|(_, r)| **r > 0.0)
        .map(|(k, r)| {
            newborn(
                cohorts[k].lineage,
                genomes.get(k).copied().unwrap_or(cohorts[k].genome),
                *r,
            )
        })
        .collect();
    for child in children {
        insert(cohorts, child);
    }
}

/// Pression de densité : au-delà de la capacité de charge (30 % de la biomasse végétale), mortalité croissante.
fn crowding(cohorts: &[Cohort], plant_kgc: f64) -> f64 {
    let animals: f64 = cohorts.iter().filter(|c| !c.is_producer()).map(Cohort::biomass).sum();
    let capacity = CARRY_FRACTION * plant_kgc;
    if capacity <= 0.0 {
        return if animals > 0.0 { CROWD_RATE } else { 0.0 };
    }
    CROWD_RATE * (animals / capacity - 1.0).max(0.0)
}

/// Bilan d'une cohorte sur un sous-pas ; renvoie la biomasse mise en réserve pour la reproduction.
fn living_step(cohort: &mut Cohort, intake: (f64, f64), ctx: &StepCtx<'_>, flux: &mut FaunaFlux) -> f64 {
    let (plant, animal) = intake;
    let assim = ASSIM_PLANT * plant + ASSIM_ANIMAL * animal;
    let before = cohort.biomass();
    let resp_raw = cohort.n * metabolic_rate_at(cohort.mass_ind, ctx.arr) * ctx.h;
    let resp = resp_raw.min(before + assim);
    flux.clamps += u32::from(resp < resp_raw);
    let surplus = assim - resp;
    let repro = if cohort.mass_ind >= cohort.mass_adult && surplus > 0.0 {
        repro_fraction(&cohort.genome) * surplus
    } else {
        0.0
    };
    let pre = (before + surplus - repro).max(0.0);
    let mass_new = if cohort.n > 0.0 { pre / cohort.n } else { 0.0 };
    let mu = mortality(cohort, (surplus, before, mass_new), ctx);
    let n_new = cohort.n * (-mu * ctx.h).exp();
    let dead = (cohort.n - n_new) * mass_new;
    flux.assim += assim;
    flux.resp += resp;
    flux.egest += plant + animal - assim;
    flux.dead += dead;
    cohort.n = n_new;
    cohort.mass_ind = mass_new;
    cohort.age_yr += ctx.h;
    repro
}

/// Mortalité annuelle : fond MTE, sénescence, famine (déficit énergétique), densité et territorialité (Damuth).
fn mortality(cohort: &Cohort, (surplus, before, mass_new): (f64, f64, f64), ctx: &StepCtx<'_>) -> f64 {
    let starvation = if surplus < 0.0 {
        STARVATION_RATE * (-surplus) / before.max(1e-300)
    } else {
        0.0
    };
    let cap = territorial_cap(mass_new, ctx.env.area_m2).max(1e-300);
    let territorial = TERRITORY_RATE * (cohort.n / cap - 1.0).max(0.0);
    mortality_at(mass_new, ctx.arr) + senescence(cohort) + starvation + ctx.crowd + territorial
}

fn senescence(cohort: &Cohort) -> f64 {
    let longevity = longevity_yr(&cohort.genome);
    (cohort.age_yr / longevity - 1.0).max(0.0) * SENESCENCE_RATE
}

/// Cohorte nouveau-née : biomasse `repro` converties en individus de masse natale.
fn newborn(lineage: u32, genome: Genome, repro: f64) -> Cohort {
    let mut child = Cohort::new(lineage, genome, 0.0);
    child.n = repro / child.mass_birth.max(1e-300);
    child.mass_ind = child.mass_birth;
    child
}
