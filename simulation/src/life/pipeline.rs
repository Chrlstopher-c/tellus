//! Pas du vivant : cellules en phase (une sur dix par tick), blocs fixes traités en parallèle, dispersion, évolution, bilan.

use rayon::prelude::*;

use super::evolution::extinction::update_extinctions;
use super::evolution::lineage::{refresh_moments, refresh_totals, LineageId};
use super::evolution::speciation::speciate;
use super::fauna::dispersal::disperse_cohorts;
use super::fauna::{remove_lineages, update_cell, Cohort, FaunaEnv, FaunaFlux};
use super::flora::biome::{surface_albedo, whittaker_class, BIOME_NONE, INLANDSIS};
use super::flora::{step_cell, FloraCell, FloraEnv, FloraFlux, LAND_PFT, PFT_COUNT};
use super::graph::Graph;
use super::io::{LifeInputs, LifeReport};
use super::ledger::Flows;
use super::regime::{evolution_probability, is_explicit};
use super::state::LifeState;
use crate::clock::Regime;
use crate::shared::grid::Grid;
use crate::shared::rng::unit_f64;

/// Une cellule sur `PHASE_PERIOD` est mise à jour à chaque tick ; son pas couvre alors `PHASE_PERIOD` × Δt.
pub const PHASE_PERIOD: u64 = 10;
/// Taille fixe des blocs : le découpage ne dépend jamais du nombre de threads.
const CHUNK_CELLS: usize = 256;
const STREAM_FIRE: u32 = 0x0F1E;
const STREAM_EVOLUTION: u32 = 0x0E70;
const ICE_THRESHOLD: f64 = 0.5;

struct CellCtx<'a> {
    inputs: &'a LifeInputs<'a>,
    graph: &'a Graph,
    area_m2: &'a [f64],
    snapshot: &'a [[f64; PFT_COUNT]],
    seed: u64,
    tick: u64,
    span: f64,
    photo: bool,
}

type ChunkJob<'a> = (usize, &'a mut [FloraCell], &'a mut [Vec<Cohort>]);

/// Avance le vivant d'un pas de `dt_years` (contrat § 12). Le résultat ne dépend pas de `state.set_threads`.
pub fn step(
    state: &mut LifeState,
    grid: &Grid,
    inputs: &LifeInputs<'_>,
    dt_years: f64,
    regime: Regime,
    tick: u64,
    seed: u64,
) -> LifeReport {
    let cells = state.cell_count();
    if grid.cell_count() != cells || !inputs.matches(cells) || !(dt_years.is_finite() && dt_years > 0.0) {
        return LifeReport::rejected();
    }
    state.last_tick = tick;
    let photo = state.has_producers();
    let snapshot: Vec<[f64; PFT_COUNT]> = state.flora.iter().map(|f| f.cover).collect();
    let mut flows = run_cells(state, inputs, &snapshot, (tick, seed, dt_years, photo));
    disperse_cohorts(
        &state.graph,
        &mut state.fauna,
        seed,
        tick,
        PHASE_PERIOD as f64 * dt_years,
        PHASE_PERIOD,
    );
    let (speciations, extinctions) = evolve(state, regime, (tick, seed, dt_years), &mut flows);
    state.ledger.apply_flows(&flows);
    state.photosynth = state.has_producers();
    update_outputs(state, inputs);
    build_report(state, flows, speciations, extinctions)
}

fn run_cells(
    state: &mut LifeState,
    inputs: &LifeInputs<'_>,
    snapshot: &[[f64; PFT_COUNT]],
    (tick, seed, dt_years, photo): (u64, u64, f64, bool),
) -> Flows {
    let pool = state.pool.clone();
    let ctx = CellCtx {
        inputs,
        graph: &state.graph,
        area_m2: &state.area_m2,
        snapshot,
        seed,
        tick,
        span: PHASE_PERIOD as f64 * dt_years,
        photo,
    };
    let jobs: Vec<ChunkJob<'_>> = state
        .flora
        .chunks_mut(CHUNK_CELLS)
        .zip(state.fauna.chunks_mut(CHUNK_CELLS))
        .enumerate()
        .map(|(k, (flora, fauna))| (k * CHUNK_CELLS, flora, fauna))
        .collect();
    run_chunks(jobs, &ctx, pool.as_deref())
        .iter()
        .fold(Flows::default(), |mut acc, part| {
            acc.add(part);
            acc
        })
}

/// Exécute les blocs en parallèle (rayon) ; les résultats sont rendus dans l'ordre des blocs : résultat identique
/// quel que soit le nombre de threads. Sans pool, les blocs sont traités séquentiellement dans le même ordre.
fn run_chunks(jobs: Vec<ChunkJob<'_>>, ctx: &CellCtx<'_>, pool: Option<&rayon::ThreadPool>) -> Vec<Flows> {
    match pool {
        Some(pool) => pool.install(move || jobs.into_par_iter().map(|job| process_chunk(job, ctx)).collect()),
        None => jobs.into_iter().map(|job| process_chunk(job, ctx)).collect(),
    }
}

fn process_chunk((base, flora, fauna): ChunkJob<'_>, ctx: &CellCtx<'_>) -> Flows {
    let mut flows = Flows::default();
    for (offset, (cell, cohorts)) in flora.iter_mut().zip(fauna.iter_mut()).enumerate() {
        let index = base + offset;
        if (index as u64 + ctx.tick).is_multiple_of(PHASE_PERIOD) {
            process_cell(index, cell, cohorts, ctx, &mut flows);
        }
    }
    flows
}

fn process_cell(index: usize, flora: &mut FloraCell, cohorts: &mut Vec<Cohort>, ctx: &CellCtx<'_>, flows: &mut Flows) {
    let to_kg = ctx.area_m2[index] / 1000.0;
    if ctx.photo {
        let env = flora_env(index, ctx);
        let flux = step_cell(flora, &env, ctx.span);
        add_flora(flows, &flux, to_kg);
    }
    let plant_kgc = flora.biomass_gc_m2.iter().sum::<f64>() * to_kg;
    let env = FaunaEnv {
        temp_c: f64::from(ctx.inputs.temperature_c[index]),
        plant_kgc,
        area_m2: ctx.area_m2[index],
        seed: ctx.seed,
        tick: ctx.tick,
        cell: index as u32,
    };
    let fauna = update_cell(cohorts, &env, ctx.span);
    consume_plants(flora, fauna.intake_plant, to_kg);
    flora.litter_gc_m2 += (fauna.egest + fauna.dead) / to_kg;
    add_fauna(flows, &fauna);
}

fn flora_env(index: usize, ctx: &CellCtx<'_>) -> FloraEnv {
    let inputs = ctx.inputs;
    let ocean = inputs.is_ocean[index];
    let f = |values: &[f32]| f64::from(values[index]);
    FloraEnv {
        temp_c: f(inputs.temperature_c),
        precip_mm: f(inputs.precip_mm_yr),
        soil_moisture: f(inputs.soil_moisture),
        npp_cap: f(inputs.npp_cap_gc_m2_yr),
        n_avail: if ocean {
            f(inputs.nutrient_ocean)
        } else {
            f(inputs.n_avail)
        },
        p_avail: f(inputs.p_avail),
        co2_ppm: inputs.co2_ppm,
        is_ocean: ocean,
        neighbor_cover: neighbour_mean(ctx, index),
        fire_u: unit_f64(ctx.seed, ctx.tick, index as u32, STREAM_FIRE),
    }
}

/// Couverture moyenne des voisins au début du pas : source de graines pour la colonisation.
fn neighbour_mean(ctx: &CellCtx<'_>, index: usize) -> [f64; PFT_COUNT] {
    let neighbours = ctx.graph.neighbors(index);
    let mut mean = [0.0; PFT_COUNT];
    if neighbours.is_empty() {
        return mean;
    }
    for &j in neighbours {
        for (m, c) in mean.iter_mut().zip(ctx.snapshot[j as usize].iter()) {
            *m += c;
        }
    }
    for m in mean.iter_mut() {
        *m /= neighbours.len() as f64;
    }
    mean
}

fn consume_plants(flora: &mut FloraCell, intake_kgc: f64, to_kg: f64) {
    let stock = flora.biomass_gc_m2.iter().sum::<f64>() * to_kg;
    if stock <= 0.0 || intake_kgc <= 0.0 {
        return;
    }
    let kept = (1.0 - intake_kgc / stock).clamp(0.0, 1.0);
    for biomass in flora.biomass_gc_m2.iter_mut() {
        *biomass *= kept;
    }
}

fn add_flora(flows: &mut Flows, flux: &FloraFlux, to_kg: f64) {
    flows.fixed += flux.fixed_gc_m2 * to_kg;
    flows.litter_in += flux.litter_in_gc_m2 * to_kg;
    flows.decomposed += flux.decomposed_gc_m2 * to_kg;
    flows.combusted += flux.combusted_gc_m2 * to_kg;
    flows.fire_cells += u32::from(flux.burned);
}

fn add_fauna(flows: &mut Flows, flux: &FaunaFlux) {
    flows.intake_plant += flux.intake_plant;
    flows.intake_animal += flux.intake_animal;
    flows.assim += flux.assim;
    flows.resp += flux.resp;
    flows.egest += flux.egest;
    flows.dead += flux.dead;
    flows.litter_in += flux.egest + flux.dead;
    flows.fusions += flux.fusions;
    flows.clamps += flux.clamps;
}

/// Spéciation, moments, extinctions. Explicite : tous les `SPECIATION_PERIOD` ticks. Relaxé : tirage p = 1 − e^(−λΔt).
fn evolve(state: &mut LifeState, regime: Regime, (tick, seed, dt): (u64, u64, f64), flows: &mut Flows) -> (u32, u32) {
    if !tick.is_multiple_of(PHASE_PERIOD) {
        return (0, 0);
    }
    let years = PHASE_PERIOD as f64 * dt;
    refresh_totals(&mut state.lineages, &state.fauna);
    let births = if speciation_due(regime, (tick, seed), years) {
        refresh_moments(&mut state.lineages, &state.fauna);
        speciate(&mut state.lineages, &mut state.fauna, &state.graph, tick, years).len()
    } else {
        0
    };
    let dead = update_extinctions(&mut state.lineages, years, tick);
    if !dead.is_empty() {
        flows.litter_in += bury_extinct(state, &dead);
    }
    (
        u32::try_from(births).unwrap_or(u32::MAX),
        u32::try_from(dead.len()).unwrap_or(u32::MAX),
    )
}

/// Retire les cohortes des lignées éteintes ; leur biomasse rejoint la litière de leur cellule.
fn bury_extinct(state: &mut LifeState, dead: &[LineageId]) -> f64 {
    let mut total = 0.0;
    for (index, cohorts) in state.fauna.iter_mut().enumerate() {
        let lost = remove_lineages(cohorts, dead);
        if lost > 0.0 {
            state.flora[index].litter_gc_m2 += lost / (state.area_m2[index] / 1000.0);
            total += lost;
        }
    }
    total
}

/// Explicite : spéciation à chaque cadence de phase. Relaxé : tirage avec p = 1 − e^(−λΔt) sur la cadence.
fn speciation_due(regime: Regime, (tick, seed): (u64, u64), years: f64) -> bool {
    is_explicit(regime) || unit_f64(seed, tick, 0, STREAM_EVOLUTION) < evolution_probability(years)
}

/// Recalcule les champs de sortie par cellule à partir de la flore et des forçages.
fn update_outputs(state: &mut LifeState, inputs: &LifeInputs<'_>) {
    let outputs = &mut state.outputs;
    for (i, flora) in state.flora.iter().enumerate() {
        let cover: f64 = flora.cover[..LAND_PFT].iter().sum::<f64>().min(1.0);
        let ocean = inputs.is_ocean[i];
        let ice = f64::from(inputs.ice_fraction[i]);
        outputs.veg_cover[i] = cover as f32;
        outputs.surface_albedo[i] = surface_albedo(cover, ice, ocean) as f32;
        outputs.biomass_gc_m2[i] = flora.biomass_gc_m2.iter().sum::<f64>() as f32;
        outputs.npp_realized_gc_m2_yr[i] = flora.npp_gc_m2_yr as f32;
        outputs.biome[i] = biome_of(inputs, i, cover);
    }
}

fn biome_of(inputs: &LifeInputs<'_>, index: usize, cover: f64) -> u8 {
    if inputs.is_ocean[index] {
        BIOME_NONE
    } else if f64::from(inputs.ice_fraction[index]) > ICE_THRESHOLD {
        INLANDSIS
    } else {
        whittaker_class(
            f64::from(inputs.temperature_c[index]),
            f64::from(inputs.precip_mm_yr[index]),
            cover,
        )
    }
}

fn build_report(state: &LifeState, flows: Flows, speciations: u32, extinctions: u32) -> LifeReport {
    let cover = state.veg_cover();
    let mean_veg_cover = if cover.is_empty() {
        0.0
    } else {
        cover.iter().map(|&c| f64::from(c)).sum::<f64>() / cover.len() as f64
    };
    LifeReport {
        input_error: false,
        photosynth_present: state.photosynth_present(),
        lineages_alive: state.lineages.alive_count(),
        mean_veg_cover,
        flora_biomass_kgc: state.flora_kgc(),
        fauna_biomass_kgc: state.fauna_kgc(),
        flows,
        speciations,
        extinctions,
        conservation_error: conservation_error(state),
    }
}

/// Écart entre les stocks du registre et l'état réel (vivant et litière), relatif à la grandeur des stocks.
fn conservation_error(state: &LifeState) -> f64 {
    let pools = state.ledger.carbon;
    let scale = 1.0 + pools.magnitude();
    let living = (state.living_kgc() - pools.living).abs();
    let litter = (state.litter_kgc() - pools.litter).abs();
    living.max(litter) / scale
}
