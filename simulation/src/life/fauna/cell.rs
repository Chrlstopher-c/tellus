//! Mise à jour animale d'une cellule : sous-pas de consommation, métabolisme, mortalité, reproduction mutée.

use super::cohort::{fuse_to_cap, insert, take_dust, Cohort};
use super::feeding::feed;
use super::metabolism::{metabolic_rate, mortality_rate};
use crate::life::evolution::{longevity_yr, mutate, repro_fraction};

const SUBSTEP_YR: f64 = 1.0;
const MAX_SUBSTEPS: u32 = 64;
const ASSIM_PLANT: f64 = 0.4;
const ASSIM_ANIMAL: f64 = 0.8;
const SENESCENCE_RATE: f64 = 0.5;
const STARVATION_RATE: f64 = 0.5;
const CROWD_RATE: f64 = 10.0;
/// Capacité de charge animale : fraction de la biomasse végétale de la cellule.
const CARRY_FRACTION: f64 = 0.02;

/// Contexte d'une cellule pour un pas : température, biomasse végétale et aléa à compteur.
#[derive(Debug, Clone, Copy)]
pub struct FaunaEnv {
    pub temp_c: f64,
    pub plant_kgc: f64,
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
    h: f64,
    crowd: f64,
}

/// Fait avancer les cohortes d'une cellule de `span` ans ; `FaunaFlux::intake_plant` est prélevé sur les plantes.
pub fn update_cell(cohorts: &mut Vec<Cohort>, env: &FaunaEnv, span: f64) -> FaunaFlux {
    let mut flux = FaunaFlux::default();
    if cohorts.is_empty() || span <= 0.0 {
        return flux;
    }
    let steps = ((span / SUBSTEP_YR).ceil() as u32).clamp(1, MAX_SUBSTEPS);
    let h = span / f64::from(steps);
    let mut plant_avail = env.plant_kgc;
    for step in 0..steps {
        substep(cohorts, env, h, step, &mut plant_avail, &mut flux);
    }
    flux.dead += take_dust(cohorts);
    flux
}

fn substep(cohorts: &mut Vec<Cohort>, env: &FaunaEnv, h: f64, step: u32, plant_avail: &mut f64, flux: &mut FaunaFlux) {
    let taken = feed(cohorts, plant_avail, h, env.temp_c);
    let ctx = StepCtx {
        env,
        h,
        crowd: crowding(cohorts, env.plant_kgc),
    };
    let mut newborn = Vec::new();
    for (k, cohort) in cohorts.iter_mut().enumerate() {
        if cohort.is_producer() || cohort.n <= 0.0 {
            continue;
        }
        let (plant, animal) = taken[k];
        flux.intake_plant += plant;
        flux.intake_animal += animal;
        let stream = step * 128 + k as u32;
        if let Some(child) = living_step(cohort, (plant, animal), &ctx, stream, flux) {
            newborn.push(child);
        }
    }
    for child in newborn {
        insert(cohorts, child);
    }
    flux.dead += take_dust(cohorts);
    flux.fusions += fuse_to_cap(cohorts);
}

/// Pression de densité : au-delà de la capacité de charge (2 % de la biomasse végétale), mortalité croissante.
fn crowding(cohorts: &[Cohort], plant_kgc: f64) -> f64 {
    let animals: f64 = cohorts.iter().filter(|c| !c.is_producer()).map(Cohort::biomass).sum();
    let capacity = CARRY_FRACTION * plant_kgc;
    if capacity <= 0.0 {
        return if animals > 0.0 { CROWD_RATE } else { 0.0 };
    }
    CROWD_RATE * (animals / capacity - 1.0).max(0.0)
}

/// Bilan d'une cohorte sur un sous-pas : assimilation, respiration, reproduction, mortalité.
fn living_step(
    cohort: &mut Cohort,
    intake: (f64, f64),
    ctx: &StepCtx<'_>,
    stream: u32,
    flux: &mut FaunaFlux,
) -> Option<Cohort> {
    let (plant, animal) = intake;
    let assim = ASSIM_PLANT * plant + ASSIM_ANIMAL * animal;
    let before = cohort.biomass();
    let resp_raw = cohort.n * metabolic_rate(cohort.mass_ind, ctx.env.temp_c) * ctx.h;
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
    (repro > 0.0).then(|| newborn(cohort, repro, ctx, stream))
}

/// Mortalité annuelle : fond MTE, sénescence, famine (déficit énergétique) et pression de densité.
fn mortality(cohort: &Cohort, (surplus, before, mass_new): (f64, f64, f64), ctx: &StepCtx<'_>) -> f64 {
    let starvation = if surplus < 0.0 {
        STARVATION_RATE * (-surplus) / before.max(1e-300)
    } else {
        0.0
    };
    mortality_rate(mass_new, ctx.env.temp_c) + senescence(cohort) + starvation + ctx.crowd
}

fn senescence(cohort: &Cohort) -> f64 {
    let longevity = longevity_yr(&cohort.genome);
    (cohort.age_yr / longevity - 1.0).max(0.0) * SENESCENCE_RATE
}

/// Cohorte nouveau-née : biomasse `repro` converties en individus de masse natale, génome muté.
fn newborn(parent: &Cohort, repro: f64, ctx: &StepCtx<'_>, stream: u32) -> Cohort {
    let genome = mutate(&parent.genome, ctx.env.seed, ctx.env.tick, ctx.env.cell, stream);
    let mut child = Cohort::new(parent.lineage, genome, 0.0);
    child.n = repro / child.mass_birth.max(1e-300);
    child.mass_ind = child.mass_birth;
    child
}
