//! État du vivant : flore et cohortes par cellule, lignées, registre, sorties par cellule. Le monde naît sans vie.

use super::evolution::{idx, Genome, LineageId, LineageTable};
use super::fauna::Cohort;
use super::flora::{FloraCell, PFT_COUNT};
use super::graph::Graph;
use super::ledger::LifeLedger;
use crate::shared::grid::Grid;
use crate::shared::rng::hash;
use std::sync::Arc;

/// Effectif fondateur d'une lignée introduite (individus par cellule).
pub const FOUNDER_N: f64 = 20.0;
const PHOTO_PRESENT_N: f64 = 1e-6;

/// Champs de sortie par cellule (noms du contrat § 12).
#[derive(Debug, Clone, Default)]
pub struct LifeOutputs {
    pub veg_cover: Vec<f32>,
    pub surface_albedo: Vec<f32>,
    pub biomass_gc_m2: Vec<f32>,
    pub npp_realized_gc_m2_yr: Vec<f32>,
    pub biome: Vec<u8>,
}

#[derive(Clone)]
pub struct LifeState {
    pub(crate) graph: Graph,
    pub(crate) area_m2: Vec<f64>,
    pub(crate) flora: Vec<FloraCell>,
    pub(crate) fauna: Vec<Vec<Cohort>>,
    pub(crate) lineages: LineageTable,
    pub(crate) ledger: LifeLedger,
    pub(crate) outputs: LifeOutputs,
    pub(crate) photosynth: bool,
    pub(crate) pool: Option<Arc<rayon::ThreadPool>>,
    pub(crate) last_tick: u64,
}

/// Monde sans vie : aucune lignée, aucune flore, aucune faune. Rien n'est ensemencé.
pub fn init(grid: &Grid) -> LifeState {
    let cells = grid.cell_count();
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let pool = build_pool(threads);
    LifeState {
        graph: Graph::from_grid(grid),
        area_m2: grid.areas_m2().to_vec(),
        flora: vec![FloraCell::default(); cells],
        fauna: vec![Vec::new(); cells],
        lineages: LineageTable::default(),
        ledger: LifeLedger::default(),
        outputs: LifeOutputs {
            veg_cover: vec![0.0; cells],
            surface_albedo: vec![0.0; cells],
            biomass_gc_m2: vec![0.0; cells],
            npp_realized_gc_m2_yr: vec![0.0; cells],
            biome: vec![0; cells],
        },
        photosynth: false,
        pool,
        last_tick: 0,
    }
}

/// Introduit une lignée fondatrice dans une cellule ; `None` si la cellule n'existe pas.
pub fn spawn_lineage(state: &mut LifeState, traits: Genome, cell: usize) -> Option<LineageId> {
    if cell >= state.fauna.len() {
        return None;
    }
    let id = state.lineages.create(traits, None, state.last_tick, cell as u32);
    let mut cohort = Cohort::new(id, traits, FOUNDER_N);
    if cohort.is_producer() {
        cohort.mass_ind = 0.0;
    }
    state.ledger.import_carbon(cohort.biomass());
    state.fauna[cell].push(cohort);
    state.photosynth = state.has_producers();
    Some(id)
}

impl LifeState {
    pub fn cell_count(&self) -> usize {
        self.fauna.len()
    }

    /// Nombre de threads utilisés par le pas. Ne change jamais le résultat (déterminisme).
    pub fn set_threads(&mut self, threads: usize) {
        self.pool = build_pool(threads);
    }

    pub fn cohorts(&self, cell: usize) -> &[Cohort] {
        &self.fauna[cell]
    }

    pub fn flora_cell(&self, cell: usize) -> &FloraCell {
        &self.flora[cell]
    }

    pub fn lineages(&self) -> &LineageTable {
        &self.lineages
    }

    pub fn ledger(&self) -> &LifeLedger {
        &self.ledger
    }

    /// Présence d'une lignée photosynthétique vivante, au dernier pas (ou à la dernière introduction).
    pub fn photosynth_present(&self) -> bool {
        self.photosynth
    }

    pub fn veg_cover(&self) -> &[f32] {
        &self.outputs.veg_cover
    }

    pub fn surface_albedo(&self) -> &[f32] {
        &self.outputs.surface_albedo
    }

    pub fn biomass_gc_m2(&self) -> &[f32] {
        &self.outputs.biomass_gc_m2
    }

    pub fn npp_realized_gc_m2_yr(&self) -> &[f32] {
        &self.outputs.npp_realized_gc_m2_yr
    }

    pub fn biome(&self) -> &[u8] {
        &self.outputs.biome
    }

    /// Biomasse carbonée vivante du monde (flore et faune, lignées photosynthétiques comprises), kgC.
    pub fn living_kgc(&self) -> f64 {
        self.flora_kgc() + self.fauna_kgc()
    }

    /// Carbone en litière, kgC.
    pub fn litter_kgc(&self) -> f64 {
        self.area_m2
            .iter()
            .zip(&self.flora)
            .map(|(area, cell)| cell.litter_gc_m2 * area / 1000.0)
            .sum()
    }

    pub fn flora_kgc(&self) -> f64 {
        self.area_m2
            .iter()
            .zip(&self.flora)
            .map(|(area, cell)| cell.biomass_gc_m2.iter().sum::<f64>() * area / 1000.0)
            .sum()
    }

    pub fn fauna_kgc(&self) -> f64 {
        self.fauna.iter().flatten().map(Cohort::biomass).sum()
    }

    /// Vrai si une lignée photosynthétique (trait 17) a au moins une cohorte vivante.
    pub(crate) fn has_producers(&self) -> bool {
        self.fauna
            .iter()
            .flatten()
            .any(|c| c.is_producer() && c.n > PHOTO_PRESENT_N)
    }

    /// Empreinte de l'état complet (bits des flottants et ordre des cellules) : test de déterminisme.
    pub fn fingerprint(&self) -> u64 {
        let mut acc = hash(0x51F7_0000, self.last_tick, 0, 0);
        for (cell, (flora, cohorts)) in self.flora.iter().zip(&self.fauna).enumerate() {
            for i in 0..PFT_COUNT {
                acc = mix(acc, flora.cover[i].to_bits(), cell as u32, i as u32);
                acc = mix(acc, flora.biomass_gc_m2[i].to_bits(), cell as u32, 16 + i as u32);
            }
            acc = mix(acc, flora.litter_gc_m2.to_bits(), cell as u32, 32);
            for c in cohorts {
                acc = mix(acc, u64::from(c.lineage), cell as u32, 64);
                acc = mix(acc, c.n.to_bits(), cell as u32, 65);
                acc = mix(acc, c.mass_ind.to_bits(), cell as u32, 66);
                acc = mix(acc, c.genome[idx::DIET].to_bits(), cell as u32, 67);
            }
        }
        mix(acc, self.lineages.count() as u64, 0, 1)
    }
}

/// Pool rayon dédié au vivant ; `None` si sa construction échoue (le pas reste alors séquentiel, même résultat).
fn build_pool(threads: usize) -> Option<Arc<rayon::ThreadPool>> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads.max(1))
        .build()
        .ok()
        .map(Arc::new)
}

fn mix(acc: u64, value: u64, cell: u32, stream: u32) -> u64 {
    hash(acc, value, cell, stream)
}
